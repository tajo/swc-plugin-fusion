import { transform } from "@swc/core";
import { format } from "prettier";
import { join } from "path";
import fs from "fs";
import * as url from "url";
import { fileURLToPath } from "url";
import { createRequire } from "module";
import * as diff from "diff";
import "colors";

const require = createRequire(import.meta.url);
const __dirname = url.fileURLToPath(new URL(".", import.meta.url));

const getSwcOpts = ({ withPlugins, pruneUnusedImports }) => ({
  isModule: true,
  filename: "code.js",
  jsc: {
    experimental: {
      ...(withPlugins
        ? { plugins: [[require.resolve("swc-plugin-fusion"), {}]] }
        : {}),
      cacheRoot: join(process.cwd(), "node_modules/.swc"),
    },
    parser: { syntax: "typescript", tsx: true, decorators: false },
  },
});

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
      const code = fs.readFileSync(join(fullPath, "code.js"), "utf-8");
      const output = await format(
        (
          await transform(
            fs.readFileSync(join(fullPath, "output.js"), "utf-8"),
            getSwcOpts({ withPlugins: false, pruneUnusedImports: false })
          )
        ).code,
        { parser: "babel" }
      );
      const config = JSON.parse(
        fs.readFileSync(join(fullPath, "config.json"), "utf-8")
      );

      // swc-plugin-fusion transform
      let result = (
        await transform(
          code,
          getSwcOpts({ withPlugins: true, pruneUnusedImports: true })
        )
      ).code;
      // manually fixing some differences caused by __dirname and __filename being different
      result = result
        .replace(
          `console.log(""); // __dirname`,
          `console.log("/path/to"); // __dirname`
        )
        .replace(/code%2Ejs/g, "%2Fpath%2Fto%2Ffile%2Ejs")
        .replace(/code\.js/g, "/path/to/file.js")
        // manual unused import pruning from the results
        // can't enable swc (powerful) pruning because it would strip the most of output code
        // and the test would get completely useless
        .replace(`import { assetUrl } from "fusion-core";\n`, "")
        .replace(`import { assetUrl as testUrl } from "fusion-core";\n`, "")
        .replace(`import { gql as test } from "fusion-plugin-apollo";\n`, "")
        .replace(`import { gql } from "fusion-plugin-apollo";\n`, "");
      result = await format(result, { parser: "babel" });

      if (result != output) {
        console.log(`Fixture ${fixtureName} failed:`);
        console.log();
        fail = true;
        const diffOutput = diff.diffChars(output, result);
        diffOutput.forEach((part) => {
          // green for additions, red for deletions
          // grey for common parts
          const color = part.added ? "green" : part.removed ? "red" : "grey";
          process.stderr.write(part.value[color]);
        });
        console.log("original");
        console.log(output);
        console.log("result");
        console.log(result);
        console.log();
      } else {
        console.log(`Fixture ${fixtureName} passed.`);
      }
    }
  }
};

run();
