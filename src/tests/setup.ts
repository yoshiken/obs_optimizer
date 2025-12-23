import '@testing-library/jest-dom';
import { expect, afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';

// テスト後のクリーンアップ
afterEach(() => {
  cleanup();
});

// Tauri APIのモック
const mockInvoke = vi.fn();
const mockListen = vi.fn();
const mockEmit = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen,
  emit: mockEmit,
}));

// グローバルモック関数をエクスポート
export { mockInvoke, mockListen, mockEmit };
