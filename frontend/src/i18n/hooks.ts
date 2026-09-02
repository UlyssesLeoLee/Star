// frontend/src/i18n/hooks.ts
// 简化 i18n hook, 阶段 1 读 zh-CN, 阶段 2 扩 en-US/ja-JP

import zhCN from './charts/zh-CN.json';

const messages: Record<string, string> = zhCN as Record<string, string>;

export function useTranslation() {
  return {
    t: (key: string, params?: Record<string, string | number>) => {
      let msg = messages[key] ?? key;
      if (params) {
        Object.entries(params).forEach(([k, v]) => {
          msg = msg.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
        });
      }
      return msg;
    },
    locale: 'zh-CN' as const,
  };
}
