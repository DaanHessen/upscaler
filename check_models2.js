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
        console.log(`Description: ${model.description}`);
        console.log(`URL: ${model.url}`);
        
        try {
            const versions = await replicate.models.versions.list(owner, name);
            if (versions && versions.results && versions.results.length > 0) {
                console.log(`Latest version: ${versions.results[0].id}`);
                console.log(`Schema:`);
                console.log(JSON.stringify(versions.results[0].openapi_schema?.components?.schemas?.Input, null, 2));
            } else {
                console.log("No versions found.");
            }
        } catch (e) {
             console.log(`Could not get versions: ${e.message}`);
             if (model.latest_version) {
                 console.log(`Latest version from model object: ${model.latest_version.id}`);
                 if (model.latest_version.openapi_schema) {
                      console.log(`Schema:`);
                      console.log(JSON.stringify(model.latest_version.openapi_schema?.components?.schemas?.Input, null, 2));
                 }
             }
        }
    } catch (e) {
        console.log(`Error fetching model ${owner}/${name}: ${e.message}`);
    }
}

async function run() {
    await checkModel("megvii-research", "nafnet");
    await checkModel("prunaai", "p-image-upscale");
    // another polish model? what about "tencent/arc2face" or "nightmareai/real-esrgan"
    await checkModel("nightmareai", "real-esrgan");
}
run().catch(console.error);