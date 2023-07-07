# swc-plugin-fusion

This is a fork of [swc-project/plugins](https://github.com/swc-project/plugins). It provides a few transformations that we use for our internal React framework Fusion. Basic overview (see the fixtures for more):

## asseturl

```js
import { assetUrl } from "fusion-core";

const Test = assetUrl("./foo.jpg");
```

into

```js
import $_asseturl___foo_jpg0 from "./foo.jpg";
import { assetUrl } from "fusion-core";
const Test = $_asseturl___foo_jpg0;
```

## dirname

```js
console.log(__dirname); // __dirname
console.log(__filename); // __filename
```

into

```js
console.log("/path/to"); // __dirname
console.log("/path/to/file.js"); // __filename
```

## gql

```js
import { gql } from "fusion-plugin-apollo";

const Test = gql("./test.gql");
```

into

```js
import $_gql___test_gql0 from "./test.gql";
import { gql } from "fusion-plugin-apollo";

const Test = $_gql___test_gql0;
```

## i18n

### Translate Component

```js
import { Translate } from "fusion-plugin-i18n-react";
export default function () {
  return (
    <>
      <span id="dont-include" />
      <Translate random={"test"} id="test" />
      <Translate id="test2" />
    </>
  );
}
```

into

```js
import _fusionPluginI18nChunkTranslationMap from "virtual:fusion-vite-i18n-map";
import { Translate } from "fusion-plugin-i18n-react";
export default function () {
  return (
    <>
      <span id="dont-include" />
      <Translate random={"test"} id="test" />
      <Translate id="test2" />
    </>
  );
}
_fusionPluginI18nChunkTranslationMap.add(
  "/path/to/file.js",
  ["vite-i18n-chunk"],
  ["test", "test2"]
);
```

### useTranslations hook

```js
import { useTranslations } from "fusion-plugin-i18n-react";

export default function () {
  const translate = useTranslations();
  translate("static");
  translate(`prefix.${"foo"}.mid.${"baz"}`);
}
```

into

```js
import _fusionPluginI18nChunkTranslationMap from "virtual:fusion-vite-i18n-map";
import { useTranslations } from "fusion-plugin-i18n-react";
export default function () {
  const translate = useTranslations();
  translate("static");
  translate(`prefix.${"foo"}.mid.${"baz"}`);
}
_fusionPluginI18nChunkTranslationMap.add(
  "/path/to/file.js",
  ["vite-i18n-chunk"],
  ["static", ["prefix.", ".mid.", ""]]
);
```

### withTranslations HOC

```js
import { withTranslations } from "fusion-plugin-i18n-react";

export default withTranslations(["test", "foo"])(
  ({ translate }) => {
    return (
      <input
        placeholder={translate("test", { name: "world" })}
      />
    );
  }
);
```

into

```js
import _fusionPluginI18nChunkTranslationMap from "virtual:fusion-vite-i18n-map";
import { withTranslations } from "fusion-plugin-i18n-react";
export default withTranslations(["test", "foo"])(
  ({ translate }) => {
    return (
      <input
        placeholder={translate("test", {
          name: "world",
        })}
      />
    );
  }
);
_fusionPluginI18nChunkTranslationMap.add(
  "/path/to/file.js",
  ["vite-i18n-chunk"],
  ["foo", "test"]
);
```

### split dynamic imports

```js
import("./foo/baz");
```

into

```js
Object.defineProperties(import("./foo/baz"), {
  __CHUNK_IDS: {
    value: [],
  },
  __MODULE_ID: {
    value:
      "virtual:fusion-vite-split-loader?importer=file.js&specifier=%2E%2Ffoo%2Fbaz",
  },
  __FUSION_DYNAMIC_IMPORT_METADATA__: {
    value: {
      version: 0,
      moduleId:
        "virtual:fusion-vite-split-loader?importer=file.js&specifier=%2E%2Ffoo%2Fbaz",
    },
  },
});
```

## Contributing

```sh
cargo build
cargo test
```
