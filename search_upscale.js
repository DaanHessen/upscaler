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
             for (let i = 0; i < Math.min(10, results.results.length); i++) {
                  const m = results.results[i];
                  console.log(`${m.owner}/${m.name} - ${m.description}`);
             }
         }
     } catch(e) {
          console.log(`Error searching for ${query}: ${e.message}`);
     }
}

async function run() {
    await searchModel("upscale");
}
run().catch(console.error);