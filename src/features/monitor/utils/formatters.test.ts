import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { formatBytes, formatPercent, formatRelativeTime, formatSpeed } from './formatters';

describe('formatters', () => {
  describe('formatBytes', () => {
    it('0バイトを正しくフォーマットする', () => {
      expect(formatBytes(0)).toBe('0 B');
    });

    it('バイト単位を正しくフォーマットする', () => {
      expect(formatBytes(500)).toBe('500.00 B');
      expect(formatBytes(1023)).toBe('1023.00 B');
    });

    it('キロバイト単位を正しくフォーマットする', () => {
      expect(formatBytes(1024)).toBe('1.00 KB');
      expect(formatBytes(1536)).toBe('1.50 KB');
      expect(formatBytes(102400)).toBe('100.00 KB');
    });

    it('メガバイト単位を正しくフォーマットする', () => {
      expect(formatBytes(1048576)).toBe('1.00 MB');
      expect(formatBytes(5242880)).toBe('5.00 MB');
      expect(formatBytes(104857600)).toBe('100.00 MB');
    });

    it('ギガバイト単位を正しくフォーマットする', () => {
      expect(formatBytes(1073741824)).toBe('1.00 GB');
      expect(formatBytes(5368709120)).toBe('5.00 GB');
    });

    it('テラバイト単位を正しくフォーマットする', () => {
      expect(formatBytes(1099511627776)).toBe('1.00 TB');
      expect(formatBytes(5497558138880)).toBe('5.00 TB');
    });

    it('小数点以下2桁まで表示する', () => {
      expect(formatBytes(1536)).toBe('1.50 KB');
      expect(formatBytes(1638400)).toBe('1.56 MB');
    });
  });

  describe('formatSpeed', () => {
    it('転送速度を正しくフォーマットする', () => {
      expect(formatSpeed(0)).toBe('0 B/s');
      expect(formatSpeed(1024)).toBe('1.00 KB/s');
      expect(formatSpeed(1048576)).toBe('1.00 MB/s');
      expect(formatSpeed(5242880)).toBe('5.00 MB/s');
    });

    it('formatBytesと同じフォーマット + /s', () => {
      const bytes = 1638400;
      expect(formatSpeed(bytes)).toBe(formatBytes(bytes) + '/s');
    });
  });

  describe('formatPercent', () => {
    it('パーセント値を正しくフォーマットする', () => {
      expect(formatPercent(0)).toBe('0.0%');
      expect(formatPercent(50)).toBe('50.0%');
      expect(formatPercent(100)).toBe('100.0%');
    });

    it('小数点以下1桁をデフォルトで表示する', () => {
      expect(formatPercent(45.5)).toBe('45.5%');
      // 四捨五入のテスト（toFixedは銀行丸めを使用する場合がある）
      const result = formatPercent(45.55);
      expect(result).toMatch(/45\.[56]%/); // 45.5% or 45.6% どちらでも可
    });

    it('小数点以下の桁数を指定できる', () => {
      expect(formatPercent(45.5, 0)).toBe('46%');
      expect(formatPercent(45.5, 1)).toBe('45.5%');
      // 小数点以下の桁数指定を確認（厳密な値は環境によって異なる可能性がある）
      expect(formatPercent(45.555, 2)).toMatch(/45\.5[56]%/);
      expect(formatPercent(45.5555, 3)).toMatch(/45\.55[56]%/);
    });

    it('整数値を正しく処理する', () => {
      expect(formatPercent(45, 0)).toBe('45%');
      expect(formatPercent(45, 2)).toBe('45.00%');
    });
  });

  describe('formatRelativeTime', () => {
    beforeEach(() => {
      // 現在時刻を固定
      vi.useFakeTimers();
      vi.setSystemTime(new Date('2024-01-01T12:00:00'));
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('1秒未満の場合は「今」と表示する', () => {
      const now = Date.now();
      expect(formatRelativeTime(now)).toBe('今');
      expect(formatRelativeTime(now - 500)).toBe('今');
      expect(formatRelativeTime(now - 999)).toBe('今');
    });

    it('1分未満の場合は「N秒前」と表示する', () => {
      const now = Date.now();
      expect(formatRelativeTime(now - 1000)).toBe('1秒前');
      expect(formatRelativeTime(now - 5000)).toBe('5秒前');
      expect(formatRelativeTime(now - 30000)).toBe('30秒前');
      expect(formatRelativeTime(now - 59000)).toBe('59秒前');
    });

    it('1時間未満の場合は「N分前」と表示する', () => {
      const now = Date.now();
      expect(formatRelativeTime(now - 60000)).toBe('1分前');
      expect(formatRelativeTime(now - 300000)).toBe('5分前');
      expect(formatRelativeTime(now - 1800000)).toBe('30分前');
      expect(formatRelativeTime(now - 3599000)).toBe('59分前');
    });

    it('1時間以上の場合は「N時間前」と表示する', () => {
      const now = Date.now();
      expect(formatRelativeTime(now - 3600000)).toBe('1時間前');
      expect(formatRelativeTime(now - 7200000)).toBe('2時間前');
      expect(formatRelativeTime(now - 86400000)).toBe('24時間前');
    });

    it('境界値を正しく処理する', () => {
      const now = Date.now();
      // 0.999秒 -> 今
      expect(formatRelativeTime(now - 999)).toBe('今');
      // 1秒 -> 1秒前
      expect(formatRelativeTime(now - 1000)).toBe('1秒前');
      // 59.999秒 -> 59秒前
      expect(formatRelativeTime(now - 59999)).toBe('59秒前');
      // 60秒 -> 1分前
      expect(formatRelativeTime(now - 60000)).toBe('1分前');
      // 59分59秒 -> 59分前
      expect(formatRelativeTime(now - 3599999)).toBe('59分前');
      // 60分 -> 1時間前
      expect(formatRelativeTime(now - 3600000)).toBe('1時間前');
    });

    it('小数点以下は切り捨てる', () => {
      const now = Date.now();
      expect(formatRelativeTime(now - 1500)).toBe('1秒前');
      expect(formatRelativeTime(now - 1999)).toBe('1秒前');
      expect(formatRelativeTime(now - 90000)).toBe('1分前');
      expect(formatRelativeTime(now - 5400000)).toBe('1時間前');
    });
  });
});
