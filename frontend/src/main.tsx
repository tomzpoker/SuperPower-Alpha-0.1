import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './styles/app.css';

const root = document.getElementById('root');
if (!root) throw new Error("Élément racine introuvable.");

createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
