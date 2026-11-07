'use client';

import { cn } from 'cn';
import { motion, useReducedMotion, type Transition } from 'motion/react';

const DIGIT_ROW_EM = 1.15;

const digitTransition: Transition = { type: 'spring', duration: 0.45, bounce: 0.1 };

type NumberToken = { kind: 'digit'; digit: number; key: string } | { kind: 'mark'; text: string; key: string };

type AnimateNumberProps = {
  children: number | string;
  prefix?: string;
  suffix?: string;
  locales?: Intl.LocalesArgument;
  format?: Intl.NumberFormatOptions;
  className?: string;
};

function AnimateNumber({ children, prefix, suffix, locales, format, className }: AnimateNumberProps) {
  const formatted = typeof children === 'number' ? formatNumber(children, locales, format) : children;
  const label = `${prefix ?? ''}${formatted}${suffix ?? ''}`;
  const reduceMotion = useReducedMotion();
  const transition = reduceMotion ? { duration: 0 } : digitTransition;
  const tokens = tokenize(formatted);

  return (
    <span
      data-slot="animate-number"
      className={cn('inline-flex items-center leading-none tabular-nums', className)}
    >
      <span className="sr-only">{label}</span>
      {prefix ? (
        <span aria-hidden className="inline-flex items-center" style={{ height: `${DIGIT_ROW_EM}em` }}>
          {prefix}
        </span>
      ) : null}
      <span aria-hidden className="inline-flex items-center">
        {tokens.map((token) =>
          token.kind === 'digit' ? (
            <DigitWheel key={token.key} digit={token.digit} transition={transition} />
          ) : (
            <span key={token.key} className="inline-flex items-center" style={{ height: `${DIGIT_ROW_EM}em` }}>
              {token.text === ' ' ? '\u00a0' : token.text}
            </span>
          )
        )}
      </span>
      {suffix ? (
        <span aria-hidden className="inline-flex items-center" style={{ height: `${DIGIT_ROW_EM}em` }}>
          {suffix}
        </span>
      ) : null}
    </span>
  );
}

function DigitWheel({ digit, transition }: { digit: number; transition: Transition }) {
  return (
    <span
      className="relative inline-block overflow-hidden"
      style={{
        width: '1ch',
        height: `${DIGIT_ROW_EM}em`,
        maskImage: 'linear-gradient(to bottom, transparent 0, #000 0.12em, #000 calc(100% - 0.12em), transparent 100%)',
      }}
    >
      <motion.span
        className="absolute inset-x-0 top-0 flex flex-col"
        initial={false}
        animate={{ y: `${-digit * DIGIT_ROW_EM}em` }}
        transition={transition}
      >
        {Array.from({ length: 10 }, (_, number) => (
          <span key={number} className="flex items-center justify-center" style={{ height: `${DIGIT_ROW_EM}em` }}>
            {number}
          </span>
        ))}
      </motion.span>
    </span>
  );
}

function tokenize(formatted: string): NumberToken[] {
  const chars = Array.from(formatted);
  let digitsFromRight = 0;
  let marksFromRight = 0;
  const tokens: NumberToken[] = [];

  for (let index = chars.length - 1; index >= 0; index -= 1) {
    const char = chars[index] ?? '';
    if (char >= '0' && char <= '9') {
      tokens.push({ kind: 'digit', digit: Number(char), key: `digit-${digitsFromRight}` });
      digitsFromRight += 1;
      continue;
    }
    tokens.push({ kind: 'mark', text: char, key: `mark-${marksFromRight}-${char}` });
    marksFromRight += 1;
  }

  tokens.reverse();
  return tokens;
}

function formatNumber(
  value: number,
  locales: Intl.LocalesArgument | undefined,
  format: Intl.NumberFormatOptions | undefined
) {
  return new Intl.NumberFormat(locales, {
    maximumFractionDigits: 0,
    useGrouping: false,
    ...format,
  }).format(value);
}

export { AnimateNumber };
export type { AnimateNumberProps };
