const Replicate = require("replicate");
const fs = require("fs");

const env = fs.readFileSync(".env", "utf8");
const token = env.match(/REPLICATE_API_TOKEN="(.+?)"/)[1];

const replicate = new Replicate({ auth: token });

async function run() {
    const output = await replicate.run(
        "shanginn/supir:7d613b6c116c06555c6c072edfa406365cd8539960f2c037022985049d4977f6",
        {
            input: {
                image: "https://replicate.delivery/pbxt/JUzE1K9o2w2nB8Yy9j5E1K9o2w2nB8Yy9/1.jpg", // any sample image?
            }
        }
    );
    console.log(output);
}
run().catch(console.error);