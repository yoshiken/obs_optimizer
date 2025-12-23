import { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';

/**
 * カスタムレンダー関数（将来的にProviderを追加する場合に備えて）
 */
function customRender(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { ...options });
}

// eslint-disable-next-line react-refresh/only-export-components
export * from '@testing-library/react';
export { customRender as render };
