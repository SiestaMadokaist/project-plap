// Web3 Smart Contract Timelock personal saving account.
// a single address for multiple purposes.


// type alias to represent solidity type in typescript
type Bytes32 = string;
type HexString = string;
type Address = `0x${HexString}`;
type TokenAddress = Address;
type Uint256 = bigint;
type Timestamp = Uint256;
type Amount = Uint256;
type Second = Uint256;
type BucketNumber = Uint256;

interface Bucket {
    token: TokenAddress;
    // purpose of this bucket (e.g: Upgrade Laptop, Liburan ke bali, etc)
    name: Bytes32;
    unlockTs: Timestamp;
    target: Amount;
    totalDeposit: Amount;
    totalWithdraw: Amount;
}

interface SavingAccount {


    /**
     * initiated when the smart contract is created.
     * all balance withdrawn will go into this address.
     */
    readonly destination: Address;

    // delegated to gnosis (?)
    readonly owner: Address;

    buckets: Bucket[];
    totalLocked: Map<TokenAddress, Amount>;
    /**
     * prevent unintended extremely high lifetime lock.
     * e.g: an unintended bucket whose lifetime was passed as javascript millisecond instead of second
     * recommended value: 100 years. (3_153_600_000)
     * initialized by constructor().block.timestamp + lifetime
     */
    readonly maxUnlockTs: Timestamp
    constructor(owner: Address, destination: Address, lifetime: Second): SavingAccount

    /**
     * @view 
     * @return token.balanceOf(self) - self.totalLocked(token)
     */
    strayAmount(token: TokenAddress): Amount;

    /**
     * @validates amount < self.strayAmount
     * @returns {Amount} sweptAmount (balance diff before and after)
     * @event [Sweep(token:TokenAddress,sweptAmount:Amount)]
     */
    sweep(token: TokenAddress, amount: Amount): Amount;


    /**
     * @owneronly
     * @param newDst 
     * this doesnt modify any bucket .unlockTs
     * all bucket still locked until its predetermined block.timestamp
     * @event [UpdateDestination(old:Address,new:Address)]
     */
    setDestination(newDst: Address): void

    /**
     * @validates msg.sender == owner || msg.sender == this.destination
     * // malas buka gnosis cuman buat initBucket baru.
     * @param {TokenAddress} token
     * limitation: only token can be saved, not native currency
     * smart contract address of the token
     * @param {Bytes32} name
     * @param {Timestamp} unlockTs 
     * this bucket can only be withdrawn after this timestamp.
     * @param {Amount} target 
     * this is only used for tracking saving achievement tracking.
     * withdrawal can still be performed even if this target is not achieved.
     * @event [InitBucket]
     */
    initBucket(token: TokenAddress, name: Bytes32, unlockTs: Timestamp, target: Amount): BucketNumber;

    /**
     * @view
     * @returns {BucketNumber} total jumlah bucket yang telah dibuat
     */
    getBucketCount(): BucketNumber;

    /**
     * @view
     * @returns {Bucket} detail informasi dari sebuah bucket
     */
    getBucket(bucket: BucketNumber): Bucket;

    /**
     * @view 
     * @return {Amount} buckets[bucket].totalDeposit - buckets[bucket].totalWithdraw 
     */
    balance(bucket: BucketNumber): Amount

    /**
     * anyone can trigger, withdraw target is still this.destination
     * @param bucket 
     * @param amount
     * amount of the bucket.currency that will be transferred from sender.address to this smart contract address
     * and locked under the bucket.
     * @returns {Amount} the current balance of this bucket (bucket.totalDeposit - bucket.totalWithdraw)
     * @event [Deposit] (bucket: BucketNumber, deposited: Amount, totalBalance: Amount)
     */
    deposit(bucket: BucketNumber, amount: Amount): Amount

    /**
     * @validates buckets[bucket].unlocked();
     * @validates isDestroyed() == false
     * anyone can trigger, withdraw target is still this.destination
     * try to withdraw all balance under this bucket.
     * make sure block.timestamp > bucket.unlockTs
     * @returns {Amount} transferred balance in this tx
     * @event [Withdraw] (bucket.name, transferred amount)
     */
    withdrawAll(bucket: BucketNumber): Amount

    /**
     * @validates buckets[bucket].unlocked();
     * @validates isDestroyed() == false
     * anyone can trigger, withdraw target is still this.destination
     * fallback if withdrawAll fails
     * (e.g: if the balance is 10.000, but the token trf mechanism burns 5% preventing the tx to be executed properly)
     * @returns {Amount} transferred balance in this tx
     * @event [Withdraw] (bucket.name, transferred amount)
     */
    withdraw(bucket: BucketNumber, amount: Amount): Amount;


    /** --- self destruct mechanism --- */
    readonly destroyedAt: Timestamp; // initiated with max(uint256);
    readonly destroyDelay: Uint256; // 7.day in second?

    /** 
     * @view
     * self.destroyedAt < block.timestamp
     */
    isDestroyed(): boolean;

    /**
     * @owneronly
     * in case owner is compromised.
     * seeing your money being frozen is emotionally more satisfying
     * than seeing your money being stolen
     * destroyedAt = block.timestamp + self.destroyDelay
     * @event [SelfDestructInitiated]
     * offchain server will track this event and send an email alarming of this event
     * with an exponential backoff starting from 1 day.
     */
    initSelfDestruct(): void

    /**
     * @validates isDestroyed() == false
     * self.destroyedAt = max(uint256)
     * @owneronly
     */
    cancelSelfDestruct(): void
}