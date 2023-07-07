import { transform } from "@swc/core";
import { format } from "prettier";
import { join } from "path";
import fs from "fs";
import * as url from "url";
import { fileURLToPath } from "url";
import { createRequire } from "module";
import * as diff from "diff";
import { transformAsync } from "@babel/core";
import "colors";

const require = createRequire(import.meta.url);
const __dirname = url.fileURLToPath(
  new URL(".", import.meta.url)
);

const run = async () => {
  const fixtures = join(__dirname, "../../fixtures");
  const dirents = fs.readdirSync(fixtures, {
    withFileTypes: true,
  });

  let fail = false;

  for (let i = 0; i < dirents.length; i++) {
    const dirent = dirents[i];
    const fullPath = join(fixtures, dirent.name);
    if (dirent.isDirectory()) {
      const fixtureName = dirent.name;
      const code = fs.readFileSync(
        join(fullPath, "code.js"),
        "utf-8"
      );
      const config = JSON.parse(
        fs.readFileSync(
          join(fullPath, "config.json"),
          "utf-8"
        )
      );

      const output = await format(
        // using babel here to remove JSX
        // can't use swc because it also removes unused imports
        (
          await transformAsync(
            fs.readFileSync(
              join(fullPath, "output.js"),
              "utf-8"
            ),
            {
              presets: ["@babel/preset-react"],
            }
          )
        ).code,
        { parser: "babel" }
      );

      // swc-plugin-fusion transform
      let result = (
        await transform(code, {
          isModule: true,
          filename: "code.js",
          jsc: {
            target: "esnext",
            experimental: {
              cacheRoot: join(
                process.cwd(),
                "node_modules/.swc"
              ),
              plugins: [
                [
                  require.resolve("swc-plugin-fusion"),
                  config,
                ],
              ],
            },
            parser: {
              syntax: "typescript",
              tsx: true,
            },
          },
        })
      ).code;
      // manually fixing some differences caused by __dirname and __filename being different
      // in the test environment
      result = result
        .replace(
          `console.log(""); // __dirname`,
          `console.log("/path/to"); // __dirname`
        )
        // .replace(/code%2Ejs/g, "%2Fpath%2Fto%2Ffile%2Ejs")
        .replace(/code\.js/g, "/path/to/file.js");
      result = await format(result, { parser: "babel" });

      if (result != output) {
        console.log(`❌ Fixture ${fixtureName} failed.`);
        console.log();
        fail = true;
        const diffOutput = diff.diffChars(output, result);
        diffOutput.forEach((part) => {
          // green for additions, red for deletions
          // grey for common parts
          const color = part.added
            ? "green"
            : part.removed
            ? "red"
            : "grey";
          process.stderr.write(part.value[color]);
        });
        console.log();
      } else {
        console.log(`✅ Fixture ${fixtureName} passed.`);
      }
    }
  }
  console.log();
  if (fail) {
    process.exit(1);
  }
  process.exit(0);
};

run();
