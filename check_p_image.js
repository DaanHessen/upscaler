const fs = require('fs');

async function run() {
    const env = fs.readFileSync('.env', 'utf8');
    const token = env.split('\n').find(l => l.startsWith('REPLICATE_API_TOKEN=')).split('=')[1].replace(/"/g, '').trim();

    const resp = await fetch("https://api.replicate.com/v1/models/prunaai/p-image-upscale/versions/9018fe338f75cea08d1e3abc5f4f795d62594abf94326d5e590090f593bb1bac", {
        headers: {
            "Authorization": `Bearer ${token}`
        }
    });
    const data = await resp.json();
    fs.writeFileSync('schema.json', JSON.stringify(data.openapi_schema.components.schemas.Input.properties, null, 2));
}

run().catch(console.error);