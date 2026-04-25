const Replicate = require("replicate");

// Monkey-patch fetch to see what URL is requested
const originalFetch = global.fetch;
global.fetch = async (url, options) => {
    console.log("FETCH URL:", url);
    console.log("FETCH BODY:", options.body);
    // Return a fake response
    return {
        ok: true,
        status: 200,
        json: async () => ({ id: "123", status: "starting" }),
        text: async () => "{}",
        headers: new Headers()
    };
};

const replicate = new Replicate({ auth: "dummy" });

const input = {
    image: "https://replicate.delivery/pbxt/MtnpGxNIVJlHAMZmQNl5bLARbYpiLahniAYis3RsRN2KwhfJ/out-1.webp",
    enhance_model: "Low Resolution V2",
    upscale_factor: "4x",
    face_enhancement: true,
    subject_detection: "Foreground",
    face_enhancement_creativity: 0.5
};

replicate.run("topazlabs/image-upscale", { input }).catch(e => console.error(e));
