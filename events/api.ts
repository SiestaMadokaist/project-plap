import dotenv from 'dotenv';
import fs from 'fs';
const { parsed } = dotenv.config({ path: "./.env.api" });
const args = {
  src: {
    provider: "s3",
    data: {
      bucket: parsed.MODEL_BUCKET,
      path: "comfy-playground",
    }
  },
  dst: {
    provider: "localhost",
    data: {
      forward: false,
      path: "comfyui/comfyui/models"
    }
  }
}
const template = {
  "path": "/agent/command/fetch",
  "httpMethod": "POST",
  body: JSON.stringify({ args, priority: Date.now() })
}

fs.writeFileSync('/tmp/api.json', JSON.stringify(template));