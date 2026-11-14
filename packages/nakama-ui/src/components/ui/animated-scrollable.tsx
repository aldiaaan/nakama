'use client';

import { cn } from 'cn';
import { AnimatePresence, MotionConfig, motion, type HTMLMotionProps, type Transition } from 'motion/react';
import {
  createContext,
  useCallback,
  useContext,
  useLayoutEffect,
  useRef,
  useState,
  type ComponentProps,
  type ComponentPropsWithoutRef,
  type ReactNode,
} from 'react';

const itemTransition: Transition = { type: 'spring', duration: 0.4, bounce: 0.1 };

type ScrollEdges = {
  scrolled: boolean;
  atBottom: boolean;
};

const AnimatedScrollableContext = createContext<((edges: ScrollEdges) => void) | null>(null);

function useAnimatedScrollableContext() {
  const setEdges = useContext(AnimatedScrollableContext);
  if (!setEdges) {
    throw new Error('AnimatedScrollable parts must be used within AnimatedScrollable.Root.');
  }
  return setEdges;
}

export type AnimatedScrollableRootProps = ComponentProps<'div'> & {
  shadowOffsetTop?: number;
  shadowOffsetBottom?: number;
};

function AnimatedScrollableRoot({
  className,
  children,
  shadowOffsetTop = 0,
  shadowOffsetBottom = 0,
  ...props
}: AnimatedScrollableRootProps) {
  const [edges, setEdges] = useState<ScrollEdges>({ scrolled: false, atBottom: true });
  const updateEdges = useCallback((next: ScrollEdges) => {
    setEdges((current) => (current.scrolled === next.scrolled && current.atBottom === next.atBottom ? current : next));
  }, []);

  return (
    <AnimatedScrollableContext.Provider value={updateEdges}>
      <div
        data-slot="animated-scrollable"
        className={cn('relative flex min-h-0 flex-col overflow-hidden rounded-[inherit]', className)}
        {...props}
      >
        {children}
        <div
          aria-hidden
          className={cn(
            'pointer-events-none absolute inset-x-0 z-10 h-6 bg-linear-to-b from-black/10 to-transparent transition-opacity dark:from-black/40',
            edges.scrolled ? 'opacity-100' : 'opacity-0'
          )}
          style={{ top: shadowOffsetTop }}
        />
        <div
          aria-hidden
          className={cn(
            'pointer-events-none absolute inset-x-0 z-10 h-12 bg-linear-to-t from-black/10 to-transparent transition-opacity dark:from-black/40',
            edges.atBottom ? 'opacity-0' : 'opacity-100'
          )}
          style={{ bottom: shadowOffsetBottom }}
        />
      </div>
    </AnimatedScrollableContext.Provider>
  );
}

export type AnimatedScrollableViewportProps = HTMLMotionProps<'div'>;

function AnimatedScrollableViewport({ className, children, onScroll, ...props }: AnimatedScrollableViewportProps) {
  const setEdges = useAnimatedScrollableContext();
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  const measure = useCallback(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }
    setEdges({
      scrolled: viewport.scrollTop > 0,
      atBottom: viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 1,
    });
  }, [setEdges]);

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    const content = contentRef.current;
    if (!viewport || !content) {
      return undefined;
    }

    measure();
    // Content grows past the viewport without resizing it, so both sizes are watched.
    const observer = new ResizeObserver(measure);
    observer.observe(viewport);
    observer.observe(content);
    return () => observer.disconnect();
  }, [measure]);

  return (
    <motion.div
      ref={viewportRef}
      data-slot="animated-scrollable-viewport"
      layoutScroll
      className={cn('min-h-0 flex-auto overflow-y-auto overscroll-contain', className)}
      onScroll={(event) => {
        measure();
        onScroll?.(event);
      }}
      {...props}
    >
      <div ref={contentRef}>{children as ReactNode}</div>
    </motion.div>
  );
}

export type AnimatedScrollableListProps = ComponentPropsWithoutRef<'ul'> & {
  as?: 'ul' | 'ol';
};

function AnimatedScrollableList({ as = 'ul', className, children, ...props }: AnimatedScrollableListProps) {
  const List = as;

  return (
    <MotionConfig reducedMotion="user">
      <List data-slot="animated-scrollable-list" className={cn('relative', className)} {...props}>
        <AnimatePresence initial={false} mode="popLayout">
          {children}
        </AnimatePresence>
      </List>
    </MotionConfig>
  );
}

export type AnimatedScrollableItemProps = HTMLMotionProps<'li'>;

function AnimatedScrollableItem({ className, ...props }: AnimatedScrollableItemProps) {
  return (
    <motion.li
      data-slot="animated-scrollable-item"
      layout
      initial={{ opacity: 0, y: -8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={itemTransition}
      className={className}
      {...props}
    />
  );
}

export const AnimatedScrollable = {
  Root: AnimatedScrollableRoot,
  Viewport: AnimatedScrollableViewport,
  List: AnimatedScrollableList,
  Item: AnimatedScrollableItem,
};
