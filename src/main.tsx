import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// 纯浏览器调试：开发模式下自动判定环境并按需安装 Tauri mock 层，
// 安装完成后再挂载 React（保证首帧前 mock 已就位）。
// 注意：不用 top-level await —— 部分 WebView（如 WebKitGTK）不支持。
// 生产构建中该分支会被静态剔除，不影响发布产物。
async function bootstrap(): Promise<void> {
  if (import.meta.env.DEV) {
    const { maybeInstallBrowserMock } = await import('./mocks/tauri-mock');
    await maybeInstallBrowserMock();
  }
  ReactDOM.createRoot(document.getElementById('root')!).render(<App />);
}

void bootstrap();