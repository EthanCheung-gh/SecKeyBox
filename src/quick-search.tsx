import React from 'react';
import ReactDOM from 'react-dom/client';
import { QuickSearchWindow } from './components/quick-search/QuickSearchWindow';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QuickSearchWindow />
  </React.StrictMode>
);