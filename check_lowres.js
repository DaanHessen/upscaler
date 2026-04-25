const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="?(.+?)"?(\n|$)/)[1].trim();

const replicate = new Replicate({ auth: token });

async function checkModel(owner, name) {
    try {
        const model = await replicate.models.get(owner, name);
        console.log(`\n=== MODEL: ${owner}/${name} ===`);
        
        let versions;
        try {
            versions = await replicate.models.versions.list(owner, name);
        } catch (e) {}

        if (versions && versions.results && versions.results.length > 0) {
            console.log(`Schema:`);
            console.log(JSON.stringify(versions.results[0].openapi_schema.components.schemas.Input, null, 2));
        } else if (model.latest_version && model.latest_version.openapi_schema) {
            console.log(`Schema:`);
            console.log(JSON.stringify(model.latest_version.openapi_schema.components.schemas.Input, null, 2));
        } else {
            console.log("No schema found.");
        }
    } catch (e) {
        console.log(`Error: ${e.message}`);
    }
}

async function run() {
    await checkModel("zsxkib", "seesr");
    await checkModel("lucataco", "supir");
}
run().catch(console.error);
