'use client';

import { IconArrowUp } from '@tabler/icons-react';
import { cn } from 'cn';
import {
  AnimatePresence,
  MotionConfig,
  motion,
  useReducedMotion,
  type HTMLMotionProps,
  type Transition,
} from 'motion/react';
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
  type RefObject,
} from 'react';

import { Button } from './button';

const itemTransition: Transition = { type: 'spring', duration: 0.4, bounce: 0.1 };
const scrollToTopTransition: Transition = { type: 'spring', duration: 0.3, bounce: 0.25 };

type ScrollEdges = {
  scrolled: boolean;
  atBottom: boolean;
};

type AnimatedScrollableContextValue = {
  edges: ScrollEdges;
  setEdges: (edges: ScrollEdges) => void;
  viewportRef: RefObject<HTMLDivElement | null>;
};

const AnimatedScrollableContext = createContext<AnimatedScrollableContextValue | null>(null);

function useAnimatedScrollableContext() {
  const context = useContext(AnimatedScrollableContext);
  if (!context) {
    throw new Error('AnimatedScrollable parts must be used within AnimatedScrollable.Root.');
  }
  return context;
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
  const viewportRef = useRef<HTMLDivElement>(null);
  const updateEdges = useCallback((next: ScrollEdges) => {
    setEdges((current) => (current.scrolled === next.scrolled && current.atBottom === next.atBottom ? current : next));
  }, []);

  return (
    <AnimatedScrollableContext.Provider value={{ edges, setEdges: updateEdges, viewportRef }}>
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
  const { setEdges, viewportRef } = useAnimatedScrollableContext();
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
  }, [setEdges, viewportRef]);

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
  }, [measure, viewportRef]);

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

export type AnimatedScrollableScrollToTopProps = ComponentProps<typeof Button>;

function AnimatedScrollableScrollToTop({
  className,
  children,
  onClick,
  variant = 'outline',
  size = 'icon-sm',
  ...props
}: AnimatedScrollableScrollToTopProps) {
  const { edges, viewportRef } = useAnimatedScrollableContext();
  const reduceMotion = useReducedMotion();

  return (
    <MotionConfig reducedMotion="user">
      <AnimatePresence initial={false}>
        {edges.scrolled ? (
          <motion.div
            key="scroll-to-top"
            className="absolute right-3 bottom-3 z-20"
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            exit={{ scale: 0 }}
            transition={scrollToTopTransition}
          >
            <Button
              data-slot="animated-scrollable-scroll-to-top"
              variant={variant}
              size={size}
              className={cn('rounded-full bg-background shadow-md hover:bg-muted', className)}
              onClick={(event) => {
                viewportRef.current?.scrollTo({ top: 0, behavior: reduceMotion ? 'auto' : 'smooth' });
                onClick?.(event);
              }}
              {...props}
            >
              {children ?? (
                <>
                  <IconArrowUp />
                  <span className="sr-only">Scroll to top</span>
                </>
              )}
            </Button>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </MotionConfig>
  );
}

export const AnimatedScrollable = {
  Root: AnimatedScrollableRoot,
  Viewport: AnimatedScrollableViewport,
  List: AnimatedScrollableList,
  Item: AnimatedScrollableItem,
  ScrollToTop: AnimatedScrollableScrollToTop,
};
