import { readFile } from "node:fs/promises";

for (const [path, exportName] of process.argv.slice(2).map((item) => {
  const split = item.lastIndexOf(":");
  if (split < 1) throw new Error(`Expected path:export-name, got ${item}`);
  return [item.slice(0, split), item.slice(split + 1)];
})) {
  const bytes = await readFile(path);
  const module = await WebAssembly.compile(bytes);
  const imports = WebAssembly.Module.imports(module);
  const instance = await WebAssembly.instantiate(module);
  const exported = instance.exports[exportName];
  if (typeof exported !== "function") throw new Error(`Missing function export ${exportName}`);
  if (imports.length !== 0) throw new Error(`${exportName} has unsupported imports: ${JSON.stringify(imports)}`);
  const result = exported(10, 20);
  if (result !== 30) throw new Error(`${exportName}(10, 20) returned ${result}, expected 30`);
  console.log(`${exportName}: ${result}; imports=${imports.length}; bytes=${bytes.length}`);
}
