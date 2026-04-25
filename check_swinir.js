const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });

async function run() {
    const versions = await replicate.models.versions.list("jingyunliang", "swinir");
    const schema = versions.results[0].openapi_schema;
    console.log(`SwinIR task_type enum: ${JSON.stringify(schema.components.schemas.task_type.enum)}`);
    console.log(`SwinIR noise enum: ${JSON.stringify(schema.components.schemas.noise?.enum)}`);
}

run().catch(console.error);