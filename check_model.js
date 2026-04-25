const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });

async function run() {
    const model = await replicate.models.get("topazlabs", "image-upscale");
    console.log(JSON.stringify(model, null, 2));
    
    // get versions
    const versions = await replicate.models.versions.list("topazlabs", "image-upscale");
    console.log(JSON.stringify(versions.results[0].openapi_schema, null, 2));
}
run().catch(console.error);