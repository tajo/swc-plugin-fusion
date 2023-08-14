import _fusionPluginI18nChunkTranslationMap from "virtual:fusion-vite-i18n-map";
import { withTranslations } from 'fusion-plugin-i18n-react';
export default withTranslations([
    'test',
    'foo'
])(({ translate })=>{
    return <input placeholder={translate('test', {
        name: 'world'
    })}/>;
});
export const Baz = compose(withReducer(baz), withTranslations([
    'bar'
])(({ translate })=>{
    return <input placeholder={translate('bar')}/>;
}));
export const Qux = compose(withTranslations([
    'qux'
]))(Q);
_fusionPluginI18nChunkTranslationMap.add("/path/to/file.js", [
    "vite-i18n-chunk"
], [
    "bar",
    "foo",
    "qux",
    "test"
]);
