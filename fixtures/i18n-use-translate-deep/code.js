import { useTranslations } from 'fusion-plugin-i18n-react';

const foo = nested(() => {
  const bar = () => {
    const translate = useTranslations();
    return {
      [hello]: translate('hello'),
    };
  };
});
