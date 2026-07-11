import { useTranslation as useTrans } from 'react-i18next';
import TranslationKeys from './TranslationKeys';

export const useTranslation = () => {
    const { t } = useTrans();
    return (key: TranslationKeys, options?: Record<string, string | number>) =>
        t(key, options) as string;
};
