import axios, { AxiosInstance } from "axios";
export interface ICivitaiClient {
    baseURL: string;
    apiKey: string;
}
export class CivitaiClient {

    constructor(private props: ICivitaiClient) { }

    request(): AxiosInstance {
        return axios.create({
            baseURL: this.props.baseURL,
            headers: {
                Authorization: `Bearer ${this.props.apiKey}`,
            }
        })
    }

    async creator(query: URLSearchParams): Promise<{ data: unknown }> {
        const resp = await this.request().get('/api/v1/creators', { params: query });
        return resp;
    }

    async models(query: URLSearchParams): Promise<{ data: unknown }> {
        const resp = await this.request().get('/api/v1/models', { params: query });
        return resp;
    }

    async modelDetail(id: number): Promise<{ data: unknown }> {
        const resp = await this.request().get(`/api/v1/models/${id}`);
        return resp;
    }

    async modelVersion(id: number): Promise<{ data: unknown }> {
        const resp = await this.request().get(`/api/v1/model-versions/${id}`);
        return resp;
    }
}

async function main(): Promise<void> {
    const client = new CivitaiClient({
        apiKey: process.env.CIVITAI_API_KEY as string,
        baseURL: process.env.CIVITAI_BASEURL as string,
    })
    // const resp = await client.creator(new URLSearchParams({
    //     username: 'makbooc',
    // }))
    // const resp = await client.modelDetail(new URLSearchParams({
    //     // query: ''
    //     ids: '3187704'
    // }));
    const detail = await client.modelVersion(3161628);
    console.log(JSON.stringify(detail.data, null, 2));
}

if (require.main === module) {
    main();
}