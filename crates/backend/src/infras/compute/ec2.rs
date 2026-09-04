use std::{net::IpAddr, time::Duration};

use aws_config::{Region, SdkConfig};
use aws_sdk_ec2::{
    error::{DisplayErrorContext, ProvideErrorMetadata},
    types::{
        Filter, Instance, InstanceLifecycleType, InstanceMarketOptionsRequest, IpPermission,
        IpRange, Ipv6Range, LaunchTemplateSpecification, MarketType, ResourceType, SecurityGroup,
        Tag, TagSpecification,
    },
    Client,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::application::ports::clients::compute::{ComputeEngine, ComputeEngines};
use domain::{
    commands::compute::{ComputeInstance, ComputeInstanceID, ComputeRegion, LaunchConfig},
    errors::DomainError,
};
use pkg::auth::claims::Username;

// tag EC2Agent::username reads back at boot via ec2:DescribeTags (infras/compute_agent/ec2.rs)
const USERNAME_TAG_KEY: &str = "Username";

// mirrors infras/compute/ec2.ts `setIPWhitelist`
const WHITELIST_SECURITY_GROUP_NAME: &str = "ec2-stable-diffusions";
// marks the ingress rules `open` owns, so it only ever touches rules it created itself
const WHITELIST_RULE_MARKER: &str = "managed-ip-whitelist";
const WHITELIST_PROTOCOLS: [&str; 2] = ["tcp", "udp"];
const WHITELIST_FROM_PORT: i32 = 0;
const WHITELIST_TO_PORT: i32 = 65535;
// let the cancelled spot request settle before terminating the instance
const SPOT_CANCEL_SETTLE: Duration = Duration::from_secs(3);

pub struct EC2 {
    region: ComputeRegion,
    client: Client,
}

pub struct EC2MultiRegion {
    config: SdkConfig,
}

impl EC2MultiRegion {
    pub fn new(config: SdkConfig) -> Self {
        EC2MultiRegion { config }
    }
}

impl ComputeEngines for EC2MultiRegion {
    type Engine = EC2;
    fn get(&self, region: &ComputeRegion) -> Option<Self::Engine> {
        // scope a client to this region the same way S3Storage does, rather than
        // reusing one client for every region
        let mut builder = self.config.to_builder();
        builder.set_region(Region::new(String::from(*region)));
        Some(EC2 {
            region: *region,
            client: Client::new(&builder.build()),
        })
    }
}

impl EC2 {
    async fn whitelist_group(&self) -> Result<SecurityGroup, DomainError> {
        let resp = self
            .client
            .describe_security_groups()
            .filters(
                Filter::builder()
                    .name("group-name")
                    .values(WHITELIST_SECURITY_GROUP_NAME)
                    .build(),
            )
            .send()
            .await
            .map_err(sdk_err)?;
        resp.security_groups()
            .iter()
            .find(|g| g.group_id().is_some())
            .cloned()
            .ok_or_else(|| {
                DomainError::Prerequisite(format!(
                    "security group not found: {WHITELIST_SECURITY_GROUP_NAME}"
                ))
            })
    }

    /// The spot request id backing `instance_id`, if this is a spot instance.
    async fn spot_request_id(&self, instance_id: &str) -> Result<Option<String>, DomainError> {
        let resp = self
            .client
            .describe_spot_instance_requests()
            .send()
            .await
            .map_err(sdk_err)?;
        let id = resp
            .spot_instance_requests()
            .iter()
            .find(|r| r.instance_id() == Some(instance_id))
            .and_then(|r| r.spot_instance_request_id())
            .map(str::to_string);
        Ok(id)
    }

    async fn revoke(
        &self,
        group_id: &str,
        permissions: Vec<IpPermission>,
    ) -> Result<(), DomainError> {
        self.client
            .revoke_security_group_ingress()
            .group_id(group_id)
            .set_ip_permissions(Some(permissions))
            .send()
            .await
            .map_err(sdk_err)?;
        Ok(())
    }
}

impl ComputeEngine for EC2 {
    fn region(&self) -> ComputeRegion {
        self.region
    }

    /// Provision one instance from `config`'s launch template, overriding its
    /// AMI with `config.image_id`, its user-data with `script` (base64-encoded,
    /// as EC2 expects), and tagging the instance with `username` so the agent
    /// that boots on it can read its own identity back via
    /// `ComputeAgent::username` (ec2:DescribeTags). ec2.ts `launch()`.
    ///
    /// `spot: true` overrides the template's market options to explicitly request
    /// spot capacity. `spot: false` leaves market options unset in this request —
    /// EC2's `MarketType` has no "on-demand" value, so there is no API-level way
    /// to force on-demand over a launch template whose own `InstanceMarketOptions`
    /// already default to spot; `false` only launches on-demand if the template
    /// itself does.
    async fn launch(
        &self,
        config: &LaunchConfig,
        username: &Username,
        script: &Option<String>,
        spot: bool,
    ) -> Result<ComputeInstance, DomainError> {
        let template = LaunchTemplateSpecification::builder()
            .launch_template_id(&config.template_id)
            .set_version(config.template_version.clone())
            .build();
        let tags = TagSpecification::builder()
            .resource_type(ResourceType::Instance)
            .tags(
                Tag::builder()
                    .key(USERNAME_TAG_KEY)
                    .value(&username.0)
                    .build(),
            )
            .build();
        tracing::info!(
            "launching from template {} in {} for {username} (spot={spot})",
            config.template_id,
            self.region
        );
        let mut builder = self
            .client
            .run_instances()
            .launch_template(template)
            .image_id(&config.image_id)
            .tag_specifications(tags);
        if spot {
            builder = builder.instance_market_options(
                InstanceMarketOptionsRequest::builder()
                    .market_type(MarketType::Spot)
                    .build(),
            );
        }
        if let Some(s) = script {
            builder = builder.user_data(STANDARD.encode(s));
        }
        let resp = builder
            .min_count(1)
            .max_count(1)
            .send()
            .await
            .map_err(sdk_err)?;
        resp.instances()
            .first()
            .and_then(to_compute_instance)
            .ok_or_else(|| DomainError::Prerequisite("run_instances returned no instance".into()))
    }

    async fn list(&self) -> Result<Vec<ComputeInstance>, DomainError> {
        let resp = self
            .client
            .describe_instances()
            .send()
            .await
            .map_err(sdk_err)?;
        let instances = resp
            .reservations()
            .iter()
            .flat_map(|r| r.instances())
            .filter_map(to_compute_instance)
            .collect();
        Ok(instances)
    }

    /// Point the shared `ec2-stable-diffusions` security group at `ip` so the
    /// caller can reach the instance. Rules this method previously added for a
    /// different address are revoked first. The instance id is unused — the grant
    /// is on the security group, matching `setIPWhitelist` in ec2.ts.
    async fn open(&self, whitelist_ip: &str) -> Result<(), DomainError> {
        let group = self.whitelist_group().await?;
        let group_id = group
            .group_id()
            .ok_or_else(|| DomainError::Prerequisite("whitelist security group has no id".into()))?
            .to_string();

        // EC2 keeps IPv4 and IPv6 CIDRs in separate fields (IpRanges vs Ipv6Ranges)
        let is_v6 = matches!(whitelist_ip.parse::<IpAddr>(), Ok(IpAddr::V6(_)));
        let target_cidr = if is_v6 {
            format!("{whitelist_ip}/128")
        } else {
            format!("{whitelist_ip}/32")
        };

        let (managed_v4, managed_v6) = collect_managed_ranges(&group);

        let stale_v4: Vec<IpPermission> = managed_v4
            .iter()
            .filter(|r| r.cidr != target_cidr)
            .map(|r| ip_permission_v4(&r.protocol, &r.cidr, None))
            .collect();
        if !stale_v4.is_empty() {
            self.revoke(&group_id, stale_v4).await?;
        }
        let stale_v6: Vec<IpPermission> = managed_v6
            .iter()
            .filter(|r| r.cidr != target_cidr)
            .map(|r| ip_permission_v6(&r.protocol, &r.cidr, None))
            .collect();
        if !stale_v6.is_empty() {
            self.revoke(&group_id, stale_v6).await?;
        }

        let managed = if is_v6 { &managed_v6 } else { &managed_v4 };
        let missing: Vec<&str> = WHITELIST_PROTOCOLS
            .iter()
            .copied()
            .filter(|p| {
                !managed
                    .iter()
                    .any(|r| r.protocol == *p && r.cidr == target_cidr)
            })
            .collect();
        if missing.is_empty() {
            return Ok(());
        }

        let permissions: Vec<IpPermission> = missing
            .iter()
            .map(|p| {
                if is_v6 {
                    ip_permission_v6(p, &target_cidr, Some(WHITELIST_RULE_MARKER))
                } else {
                    ip_permission_v4(p, &target_cidr, Some(WHITELIST_RULE_MARKER))
                }
            })
            .collect();

        tracing::info!("whitelisting {target_cidr} on {group_id}");
        match self
            .client
            .authorize_security_group_ingress()
            .group_id(&group_id)
            .set_ip_permissions(Some(permissions))
            .send()
            .await
        {
            Ok(_) => Ok(()),
            // another caller already added the same rule between our read and write
            Err(e) if e.as_service_error().and_then(|se| se.code()) == DUPLICATE_PERMISSION => {
                Ok(())
            }
            Err(e) => Err(sdk_err(e)),
        }
    }

    async fn stop(&self, id: &ComputeInstanceID) -> Result<(), DomainError> {
        tracing::info!("stopping instance {} in {}", id, self.region);
        self.client
            .stop_instances()
            .instance_ids(id.0.clone())
            .send()
            .await
            .map_err(sdk_err)?;
        Ok(())
    }

    /// ec2.ts `launch()` provisions a fresh spot instance; here the id argument
    /// means we only bring an existing, stopped instance back up (ec2.ts `start()`).
    async fn start(&self, id: &ComputeInstanceID) -> Result<(), DomainError> {
        tracing::info!("starting instance {} in {}", id, self.region);
        self.client
            .start_instances()
            .instance_ids(id.0.clone())
            .send()
            .await
            .map_err(sdk_err)?;
        Ok(())
    }

    async fn terminate(&self, id: &ComputeInstanceID) -> Result<(), DomainError> {
        // a spot instance relaunches itself while its request is active — cancel
        // the request first, then let it settle before terminating (ec2.ts)
        if let Some(request_id) = self.spot_request_id(&id.0).await? {
            tracing::info!("cancelling spot request {request_id} for {id}");
            self.client
                .cancel_spot_instance_requests()
                .spot_instance_request_ids(request_id)
                .send()
                .await
                .map_err(sdk_err)?;
            tokio::time::sleep(SPOT_CANCEL_SETTLE).await;
        }
        tracing::info!("terminating instance {} in {}", id, self.region);
        self.client
            .terminate_instances()
            .instance_ids(id.0.clone())
            .send()
            .await
            .map_err(sdk_err)?;
        Ok(())
    }

    async fn reboot(&self, id: &ComputeInstanceID) -> Result<(), DomainError> {
        tracing::info!("rebooting instance {} in {}", id, self.region);
        self.client
            .reboot_instances()
            .instance_ids(id.0.clone())
            .send()
            .await
            .map_err(sdk_err)?;
        Ok(())
    }
}

const DUPLICATE_PERMISSION: Option<&str> = Some("InvalidPermission.Duplicate");

fn to_compute_instance(i: &Instance) -> Option<ComputeInstance> {
    let id = i.instance_id()?.to_string();
    let ip = i.public_ip_address().and_then(|x| x.parse::<IpAddr>().ok());
    Some(ComputeInstance {
        ip,
        id: ComputeInstanceID(id),
        is_spot: i.instance_lifecycle() == Some(&InstanceLifecycleType::Spot),
        status: i
            .state()
            .and_then(|s| s.name())
            .map(|n| n.as_str().to_owned())
            .unwrap_or_else(|| "unknown".to_owned()),
        tipe: i
            .instance_type()
            .map(|t| t.as_str().to_owned())
            .unwrap_or_default(),
    })
}

/// An ingress CIDR this module owns (tagged with [`WHITELIST_RULE_MARKER`]),
/// paired with the protocol of the rule it lives on.
struct ManagedRange {
    protocol: String,
    cidr: String,
}

fn collect_managed_ranges(group: &SecurityGroup) -> (Vec<ManagedRange>, Vec<ManagedRange>) {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    for perm in group.ip_permissions() {
        let Some(protocol) = perm.ip_protocol() else {
            continue;
        };
        for range in perm.ip_ranges() {
            if range.description() == Some(WHITELIST_RULE_MARKER) {
                if let Some(cidr) = range.cidr_ip() {
                    v4.push(ManagedRange {
                        protocol: protocol.to_string(),
                        cidr: cidr.to_string(),
                    });
                }
            }
        }
        for range in perm.ipv6_ranges() {
            if range.description() == Some(WHITELIST_RULE_MARKER) {
                if let Some(cidr) = range.cidr_ipv6() {
                    v6.push(ManagedRange {
                        protocol: protocol.to_string(),
                        cidr: cidr.to_string(),
                    });
                }
            }
        }
    }
    (v4, v6)
}

fn ip_permission_v4(protocol: &str, cidr: &str, description: Option<&str>) -> IpPermission {
    let mut range = IpRange::builder().cidr_ip(cidr);
    if let Some(d) = description {
        range = range.description(d);
    }
    IpPermission::builder()
        .ip_protocol(protocol)
        .from_port(WHITELIST_FROM_PORT)
        .to_port(WHITELIST_TO_PORT)
        .ip_ranges(range.build())
        .build()
}

fn ip_permission_v6(protocol: &str, cidr: &str, description: Option<&str>) -> IpPermission {
    let mut range = Ipv6Range::builder().cidr_ipv6(cidr);
    if let Some(d) = description {
        range = range.description(d);
    }
    IpPermission::builder()
        .ip_protocol(protocol)
        .from_port(WHITELIST_FROM_PORT)
        .to_port(WHITELIST_TO_PORT)
        .ipv6_ranges(range.build())
        .build()
}

/// `SdkError`'s own `Display` is a terse constant per variant (e.g. "service
/// error") with no service-provided detail — the actual AccessDenied/etc.
/// message only shows up by walking the `source()` chain, which
/// `DisplayErrorContext` does.
fn sdk_err<E>(e: E) -> DomainError
where
    E: std::error::Error,
{
    DomainError::ApiError(DisplayErrorContext(e).to_string())
}
