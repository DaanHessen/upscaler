const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });
const DUMMY_IMAGE = "https://replicate.delivery/pbxt/KWDkejqLfER3jrroDTUsSvBWFaHtapPxfg4xxZIqYmfh3zXm/Screenshot%202024-02-28%20at%2022.14.00.png";

async function run() {
    console.log("Testing SwinIR...");
    try {
        const output = await replicate.run(
            "jingyunliang/swinir:660d922d33153019e8c263a3bba265de882e7f4f70396546b6c9c8f9d47a021a",
            {
                input: {
                    image: DUMMY_IMAGE,
                    task_type: "JPEG Compression Artifact Reduction",
                    jpeg: 40
                }
            }
        );
        console.log("SwinIR Output:", output);
    } catch(e) { console.error("SwinIR Error:", e.message); }
}

run().catch(console.error);