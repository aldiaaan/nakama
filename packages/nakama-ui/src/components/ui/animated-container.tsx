'use client';

import { cn } from 'cn';
import { motion, useReducedMotion } from 'motion/react';
import { useEffect, useLayoutEffect, useRef, useState, type ComponentProps, type ReactNode } from 'react';

const sizeEase: [number, number, number, number] = [0.25, 1, 0.5, 1];

export type AnimatedContainerProps = {
  as?: 'div' | 'span';
  children: ReactNode;
} & Omit<
  ComponentProps<'div'>,
  'children' | 'onAnimationEnd' | 'onAnimationStart' | 'onDrag' | 'onDragEnd' | 'onDragStart'
>;

export function AnimatedContainer({ as = 'div', children, className, style, ...props }: AnimatedContainerProps) {
  const reduceMotion = useReducedMotion();
  const surfaceRef = useRef<HTMLElement>(null);
  const measureDivRef = useRef<HTMLDivElement>(null);
  const measureSpanRef = useRef<HTMLSpanElement>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(null);
  const [animateSize, setAnimateSize] = useState(false);
  const Component = (as === 'span' ? motion.span : motion.div) as typeof motion.div;

  useLayoutEffect(() => {
    const surface = surfaceRef.current;
    const element = as === 'span' ? measureSpanRef.current : measureDivRef.current;

    if (!surface || !element) {
      return undefined;
    }

    let active = true;
    const update = () => {
      if (!active) {
        return;
      }

      // The animated width would otherwise lock the measurement to the previous size.
      const width = surface.style.width;
      const height = surface.style.height;
      surface.style.width = 'auto';
      surface.style.height = 'auto';
      const nextWidth = surface.offsetWidth;
      const nextHeight = surface.offsetHeight;
      surface.style.width = width;
      surface.style.height = height;

      if (nextWidth <= 0 || nextHeight <= 0) {
        return;
      }

      setSize((current) =>
        current?.width === nextWidth && current.height === nextHeight
          ? current
          : { width: nextWidth, height: nextHeight }
      );
    };

    update();
    const observer = new ResizeObserver(update);
    observer.observe(element);

    return () => {
      active = false;
      observer.disconnect();
    };
  }, [as]);

  useEffect(() => {
    if (size === null || animateSize) {
      return;
    }

    // The first measurement snaps. Later content changes ease the box.
    setAnimateSize(true);
  }, [animateSize, size]);

  return (
    <Component
      {...props}
      ref={(node) => {
        surfaceRef.current = node;
      }}
      data-slot="animated-container"
      className={cn(as === 'span' && 'inline-block', 'overflow-hidden', className)}
      style={style}
      initial={false}
      animate={size === null ? { width: 'auto', height: 'auto' } : { width: size.width, height: size.height }}
      transition={{
        duration: animateSize && reduceMotion !== true ? 0.27 : 0,
        ease: sizeEase,
      }}
    >
      {as === 'span' ? (
        <span ref={measureSpanRef} className="inline-block">
          {children}
        </span>
      ) : (
        <div ref={measureDivRef} className="mx-auto w-fit max-w-full">
          {children}
        </div>
      )}
    </Component>
  );
}
