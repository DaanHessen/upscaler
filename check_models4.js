const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });

async function checkModel(owner, name) {
    try {
        const model = await replicate.models.get(owner, name);
        console.log(`\n=== MODEL: ${owner}/${name} ===`);
        console.log(`License: ${model.license_url || 'Unknown'}`);
        try {
            const versions = await replicate.models.versions.list(owner, name);
            if (versions && versions.results && versions.results.length > 0) {
                console.log(`Schema:`);
                console.log(JSON.stringify(versions.results[0].openapi_schema?.components?.schemas?.Input, null, 2));
            }
        } catch (e) {
             if (model.latest_version && model.latest_version.openapi_schema) {
                  console.log(`Schema:`);
                  console.log(JSON.stringify(model.latest_version.openapi_schema?.components?.schemas?.Input, null, 2));
             }
        }
    } catch (e) {}
}

async function run() {
    await checkModel("cszn", "scunet");
    await checkModel("jingyunliang", "swinir");
}
run().catch(console.error);