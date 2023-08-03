import _fusionPluginI18nChunkTranslationMap from "virtual:fusion-vite-i18n-map";
import { Translate } from "fusion-plugin-i18n-react";
const Foo = <Translate id="foo" />;
_fusionPluginI18nChunkTranslationMap.add("/path/to/file.js", [
  "vite-i18n-chunk"
], [
  "foo"
]);
