import { Translate } from "fusion-plugin-i18n-react";

const Foo = <Translate id="foo" />;

const Bar = <Foo renderProp={() => <Translate id="bar" />} />
