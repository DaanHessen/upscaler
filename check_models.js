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
                console.log(JSON.stringify(versions.results[0].openapi_schema.components.schemas.Input, null, 2));
            } else {
                console.log("No versions found.");
            }
        } catch (e) {
             console.log(`Could not get versions: ${e.message}`);
             if (model.latest_version) {
                 console.log(`Latest version from model object: ${model.latest_version.id}`);
                 if (model.latest_version.openapi_schema) {
                      console.log(`Schema:`);
                      console.log(JSON.stringify(model.latest_version.openapi_schema.components.schemas.Input, null, 2));
                 }
             }
        }
    } catch (e) {
        console.log(`Error fetching model ${owner}/${name}: ${e.message}`);
    }
}

async function searchModel(query) {
     try {
         const results = await replicate.models.search(query);
         console.log(`\n=== SEARCH RESULTS FOR: ${query} ===`);
         if (results && results.results) {
             for (let i = 0; i < Math.min(3, results.results.length); i++) {
                  const m = results.results[i];
                  console.log(`${m.owner}/${m.name} - ${m.description}`);
             }
         }
     } catch(e) {
          console.log(`Error searching for ${query}: ${e.message}`);
     }
}

async function run() {
    await searchModel("nafnet");
    await searchModel("pruna");
    // Some known ones:
    // mv-lab/nafnet
    // prunaai/p-image-upscale
}
run().catch(console.error);