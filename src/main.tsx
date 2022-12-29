import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './components/App';

const container = document.getElementById('container');
const root = createRoot(container);

root.render(
    <StrictMode>
        <App />
    </StrictMode>,
);
