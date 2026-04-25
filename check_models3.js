const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });

async function searchModel(query) {
     try {
         const results = await replicate.models.search(query);
         console.log(`\n=== SEARCH RESULTS FOR: ${query} ===`);
         if (results && results.results) {
             for (let i = 0; i < Math.min(5, results.results.length); i++) {
                  const m = results.results[i];
                  console.log(`${m.owner}/${m.name} - ${m.description} (License: ${m.license_url})`);
             }
         }
     } catch(e) {
          console.log(`Error searching for ${query}: ${e.message}`);
     }
}

async function run() {
    await searchModel("scunet");
    await searchModel("denoise");
    await searchModel("restore");
    await searchModel("sharpen");
    
    // Also let's print nafnet's task_type options to be sure
    const model = await replicate.models.get("megvii-research", "nafnet");
    const versions = await replicate.models.versions.list("megvii-research", "nafnet");
    console.log(`\nNAFNet task_type enum: ${JSON.stringify(versions.results[0].openapi_schema.components.schemas.task_type.enum)}`);
}
run().catch(console.error);