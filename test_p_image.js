const fs = require('fs');

async function testPImage() {
    const env = fs.readFileSync('.env', 'utf8');
    const token = env.split('\n').find(l => l.startsWith('REPLICATE_API_TOKEN=')).split('=')[1].replace(/"/g, '').trim();

    const imageUrl = "https://raw.githubusercontent.com/octocat/Spoon-Knife/master/README.md"; // wait, need an image.
    const validImg = "https://avatars.githubusercontent.com/u/583231?v=4"; // Octocat
    
    const resp = await fetch("https://api.replicate.com/v1/predictions", {
        method: "POST",
        headers: {
            "Authorization": `Bearer ${token}`,
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            version: "9018fe338f75cea08d1e3abc5f4f795d62594abf94326d5e590090f593bb1bac",
            input: {
                image: validImg,
                target: 1, // 1 megapixel
                upscale_mode: "target",
                enhance_details: true,
                enhance_realism: true
            }
        })
    });
    
    let pred = await resp.json();
    if (!pred.id) { console.log(pred); return; }
    
    while (pred.status === "starting" || pred.status === "processing") {
        await new Promise(r => setTimeout(r, 1000));
        const check = await fetch(`https://api.replicate.com/v1/predictions/${pred.id}`, {
            headers: {
                "Authorization": `Bearer ${token}`
            }
        });
        pred = await check.json();
        console.log("Status:", pred.status);
    }
    
    console.log("Error:", pred.error);
    console.log("Output:", pred.output);
}

testPImage().catch(console.error);