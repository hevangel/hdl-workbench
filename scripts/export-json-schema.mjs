import { mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { WireSchemas } from "../packages/protocol/dist/index.js";

const outputDirectory = fileURLToPath(new URL("../packages/protocol/schemas/", import.meta.url));
await mkdir(outputDirectory, { recursive: true });

for (const [name, schema] of Object.entries(WireSchemas)) {
  const jsonSchema = {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    ...schema.toJSONSchema({ target: "draft-2020-12" }),
  };
  await writeFile(`${outputDirectory}${name}.schema.json`, `${JSON.stringify(jsonSchema, null, 2)}\n`, "utf8");
}
