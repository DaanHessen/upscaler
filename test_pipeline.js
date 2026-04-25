const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });
const DUMMY_IMAGE = "https://replicate.delivery/pbxt/KWDkejqLfER3jrroDTUsSvBWFaHtapPxfg4xxZIqYmfh3zXm/Screenshot%202024-02-28%20at%2022.14.00.png";

async function run() {
    console.log("Testing NAFNet...");
    try {
        const output = await replicate.run(
            "megvii-research/nafnet:018241a6c880319404eaa2714b764313e27e11f950a7ff0a7b5b37b27b74dcf7",
            {
                input: {
                    image: DUMMY_IMAGE,
                    task_type: "Image Debluring (REDS)"
                }
            }
        );
        console.log("NAFNet Output:", output);
    } catch(e) { console.error("NAFNet Error:", e.message); }

    console.log("Testing Topaz...");
    try {
        const output = await replicate.run(
            "topazlabs/image-upscale",
            {
                input: {
                    image: DUMMY_IMAGE,
                    enhance_model: "Standard V2",
                    upscale_factor: "2x",
                    face_enhancement: false,
                    subject_detection: "None"
                }
            }
        );
        console.log("Topaz Output:", output);
    } catch(e) { console.error("Topaz Error:", e.message); }

    console.log("Testing SCUNet...");
    try {
        const output = await replicate.run(
            "cszn/scunet:b4eb5b1db3c94294246d628d09559c55b6ef2dd33c5eeb24f2b1d9fc665ed5b7",
            {
                input: {
                    image: DUMMY_IMAGE,
                    model_name: "real image denoising"
                }
            }
        );
        console.log("SCUNet Output:", output);
    } catch(e) { console.error("SCUNet Error:", e.message); }
}

run().catch(console.error);