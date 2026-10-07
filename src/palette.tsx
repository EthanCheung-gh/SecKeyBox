import React from 'react';
import ReactDOM from 'react-dom/client';
import { PaletteWindow } from './components/palette/PaletteWindow';
import './index.css';

// 纯浏览器调试：开发模式下且不在真实 Tauri 环境中时安装 mock。
async function bootstrap(): Promise<void> {
  if (import.meta.env.DEV) {
    const { maybeInstallBrowserMock } = await import('./mocks/tauri-mock');
    await maybeInstallBrowserMock();
  }
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <PaletteWindow />
    </React.StrictMode>
  );
}

void bootstrap();
