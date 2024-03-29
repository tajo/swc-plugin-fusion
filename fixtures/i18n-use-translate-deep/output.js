import _fusionPluginI18nChunkTranslationMap from "virtual:fusion-vite-i18n-map";
import { useTranslations } from 'fusion-plugin-i18n-react';
const foo = nested(() => {
  const bar = () => {
    const translate = useTranslations();
    return {
      [hello]: translate('hello')
    };
  };
});
_fusionPluginI18nChunkTranslationMap.add("/path/to/file.js", [
  "vite-i18n-chunk"
], [
  "hello"
]);
