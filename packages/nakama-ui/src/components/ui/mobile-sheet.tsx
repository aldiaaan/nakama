'use client';

import { Drawer as DrawerPrimitive } from '@base-ui/react/drawer';
import { IconX } from '@tabler/icons-react';
import { cn } from 'cn';
import { AnimatePresence, motion, useReducedMotion } from 'motion/react';
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ComponentProps,
  type ReactNode,
} from 'react';

import { Button } from './button';

export type MobileSheetRoute = {
  id: string;
  title?: string;
  element: ReactNode;
};

export type MobileSheetNavigation = {
  id: string;
  depth: number;
  canBack: boolean;
  push: (id: string) => void;
  replace: (id: string) => void;
  back: () => void;
  reset: () => void;
};

type MobileSheetDirection = 'push' | 'pop';

type MobileSheetContextValue = {
  modal: NonNullable<DrawerPrimitive.Root.Props['modal']>;
  open: boolean;
  layoutKey: number;
  routes: readonly MobileSheetRoute[];
  direction: MobileSheetDirection;
  navigation: MobileSheetNavigation;
};

type StackState = {
  stack: string[];
  direction: MobileSheetDirection;
};

const MobileSheetContext = createContext<MobileSheetContextValue | null>(null);

const screenEase: [number, number, number, number] = [0.26, 0.08, 0.25, 1];
const heightEase: [number, number, number, number] = [0.25, 1, 0.5, 1];

function screenTransition(direction: MobileSheetDirection, reduceMotion: boolean) {
  return {
    duration: reduceMotion ? 0 : direction === 'pop' ? 0.22 : 0.27,
    ease: screenEase,
  };
}

function assertRoute(routes: readonly MobileSheetRoute[], id: string) {
  if (!routes.some((route) => route.id === id)) {
    throw new Error(`Unknown mobile sheet route: ${id}`);
  }
}

function useMobileSheetContext() {
  const context = useContext(MobileSheetContext);

  if (!context) {
    throw new Error('useMobileSheet must be used within a MobileSheet.');
  }

  return context;
}

export function useMobileSheet(): MobileSheetNavigation {
  return useMobileSheetContext().navigation;
}

export type MobileSheetRootProps = Omit<DrawerPrimitive.Root.Props, 'children' | 'swipeDirection'> & {
  routes: readonly MobileSheetRoute[];
  initialId: string;
  children: ReactNode;
};

function MobileSheetRoot({
  routes,
  initialId,
  children,
  modal = true,
  open: openProp,
  defaultOpen = false,
  onOpenChange,
  onOpenChangeComplete,
  ...props
}: MobileSheetRootProps) {
  assertRoute(routes, initialId);
  const [uncontrolledOpen, setUncontrolledOpen] = useState(defaultOpen);
  const [layoutKey, setLayoutKey] = useState(0);
  const [state, setState] = useState<StackState>({ stack: [initialId], direction: 'push' });
  const open = openProp ?? uncontrolledOpen;

  const push = useCallback(
    (id: string) => {
      assertRoute(routes, id);
      setState((current) => ({ direction: 'push', stack: [...current.stack, id] }));
    },
    [routes]
  );

  const replace = useCallback(
    (id: string) => {
      assertRoute(routes, id);
      setState((current) => ({
        direction: 'push',
        stack: [...current.stack.slice(0, -1), id],
      }));
    },
    [routes]
  );

  const back = useCallback(() => {
    setState((current) => {
      if (current.stack.length <= 1) {
        return current;
      }

      return { direction: 'pop', stack: current.stack.slice(0, -1) };
    });
  }, []);

  const reset = useCallback(() => {
    setState({ stack: [initialId], direction: 'pop' });
  }, [initialId]);

  const id = state.stack[state.stack.length - 1] ?? initialId;
  const navigation = useMemo<MobileSheetNavigation>(
    () => ({
      id,
      depth: state.stack.length,
      canBack: state.stack.length > 1,
      push,
      replace,
      back,
      reset,
    }),
    [back, id, push, replace, reset, state.stack.length]
  );

  const contextValue = useMemo<MobileSheetContextValue>(
    () => ({
      modal,
      open,
      layoutKey,
      routes,
      direction: state.direction,
      navigation,
    }),
    [layoutKey, modal, navigation, open, routes, state.direction]
  );

  return (
    <MobileSheetContext.Provider value={contextValue}>
      <DrawerPrimitive.Root
        data-slot="mobile-sheet"
        {...props}
        {...(openProp === undefined ? { defaultOpen } : { open: openProp })}
        modal={modal}
        swipeDirection="down"
        onOpenChange={(next, details) => {
          setUncontrolledOpen(next);
          onOpenChange?.(next, details);
        }}
        onOpenChangeComplete={(next) => {
          if (!next) {
            setState({ stack: [initialId], direction: 'push' });
            setLayoutKey((key) => key + 1);
          }
          onOpenChangeComplete?.(next);
        }}
      >
        {children}
      </DrawerPrimitive.Root>
    </MobileSheetContext.Provider>
  );
}

function MobileSheetTrigger({ ...props }: DrawerPrimitive.Trigger.Props) {
  return <DrawerPrimitive.Trigger data-slot="mobile-sheet-trigger" {...props} />;
}

export type MobileSheetContentProps = Omit<DrawerPrimitive.Popup.Props, 'children'> & {
  title: string;
  description?: string;
  showCloseButton?: boolean;
};

function MobileSheetContent({
  title,
  description,
  className,
  showCloseButton = true,
  ...props
}: MobileSheetContentProps) {
  const { modal, layoutKey } = useMobileSheetContext();

  return (
    <DrawerPrimitive.Portal>
      {modal === true ? <MobileSheetOverlay /> : null}
      <DrawerPrimitive.Viewport
        data-slot="mobile-sheet-viewport"
        className="pointer-events-none fixed inset-0 z-50 flex items-end justify-center p-4"
      >
        <DrawerPrimitive.Popup
          data-slot="mobile-sheet-popup"
          className={cn(
            'pointer-events-auto relative w-full max-w-90.25 shrink-0 rounded-[36px] bg-popover text-sm text-popover-foreground shadow-lg ring-1 ring-foreground/10 outline-none',
            'transform-[translate3d(0,var(--translate-y,0px),0)] transition-transform duration-450 ease-[cubic-bezier(0.32,0.72,0,1)] will-change-transform motion-reduce:transition-none',
            'data-ending-style:transform-(--closed-transform) data-starting-style:transform-(--closed-transform) data-swiping:duration-0 data-ending-style:duration-[calc(var(--drawer-swipe-strength)*400ms)]',
            '[--closed-transform:translate3d(0,calc(100%+1rem),0)] [--translate-y:var(--drawer-swipe-movement-y,0px)]',
            className
          )}
          {...props}
        >
          {description ? (
            <DrawerPrimitive.Description className="sr-only">{description}</DrawerPrimitive.Description>
          ) : null}
          <MobileSheetSurface key={layoutKey} showCloseButton={showCloseButton} title={title} />
        </DrawerPrimitive.Popup>
      </DrawerPrimitive.Viewport>
    </DrawerPrimitive.Portal>
  );
}

function MobileSheetSurface({ showCloseButton, title }: { showCloseButton: boolean; title: string }) {
  const { open, routes, direction, navigation } = useMobileSheetContext();
  const reduceMotion = useReducedMotion();
  const surfaceRef = useRef<HTMLDivElement>(null);
  const measureRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState<number | null>(null);
  const [animateHeight, setAnimateHeight] = useState(false);
  const route = routes.find((item) => item.id === navigation.id);
  // The exiting screen calls these variants after it leaves the tree, so they read the latest direction.
  const directionRef = useRef(direction);
  directionRef.current = direction;
  const reduceMotionRef = useRef(reduceMotion === true);
  reduceMotionRef.current = reduceMotion === true;
  const screenVariants = useMemo(
    () => ({
      hidden: () => ({
        opacity: 0,
        scale: 0.96,
        transition: screenTransition(directionRef.current, reduceMotionRef.current),
      }),
      visible: () => ({
        opacity: 1,
        scale: 1,
        transition: screenTransition(directionRef.current, reduceMotionRef.current),
      }),
    }),
    []
  );

  useLayoutEffect(() => {
    if (!open) {
      return undefined;
    }

    const surface = surfaceRef.current;
    const element = measureRef.current;

    if (!surface || !element) {
      return undefined;
    }

    let active = true;
    const update = () => {
      if (!active) {
        return;
      }

      // The surface's animated height would otherwise report the previous screen's size.
      const locked = surface.style.height;
      if (locked && locked !== 'auto') {
        surface.style.height = 'auto';
      }
      const next = element.offsetHeight;
      if (locked && locked !== 'auto') {
        surface.style.height = locked;
      }

      if (next <= 0) {
        return;
      }

      setHeight((current) => (current === next ? current : next));
    };

    update();
    const observer = new ResizeObserver(update);
    observer.observe(element);

    return () => {
      active = false;
      observer.disconnect();
    };
  }, [navigation.depth, navigation.id, open]);

  useEffect(() => {
    if (height === null || animateHeight) {
      return;
    }

    // The first measurement snaps. Later route changes ease the sheet height.
    setAnimateHeight(true);
  }, [animateHeight, height]);

  if (!route) {
    throw new Error(`Unknown mobile sheet route: ${navigation.id}`);
  }

  return (
    <motion.div
      ref={surfaceRef}
      className="overflow-hidden rounded-[inherit]"
      initial={false}
      animate={{ height: height ?? 'auto' }}
      transition={{
        duration: animateHeight && reduceMotion !== true ? 0.27 : 0,
        ease: heightEase,
      }}
    >
      <div ref={measureRef}>
        <div className="flex items-center justify-between gap-3 px-6 pt-4">
          <DrawerPrimitive.Title className="min-w-0 flex-1 text-base font-medium">
            {route.title ?? title}
          </DrawerPrimitive.Title>
          {showCloseButton ? <MobileSheetClose /> : null}
        </div>
        <AnimatePresence initial={false} mode="popLayout">
          <motion.div
            key={`${navigation.id}:${navigation.depth}`}
            className="w-full px-6 pt-3 pb-6"
            variants={screenVariants}
            initial="hidden"
            animate="visible"
            exit="hidden"
          >
            <DrawerPrimitive.Content data-slot="mobile-sheet-screen">{route.element}</DrawerPrimitive.Content>
          </motion.div>
        </AnimatePresence>
      </div>
    </motion.div>
  );
}

function MobileSheetOverlay({ className, ...props }: DrawerPrimitive.Backdrop.Props) {
  return (
    <DrawerPrimitive.Backdrop
      data-slot="mobile-sheet-overlay"
      className={cn(
        'fixed inset-0 z-50 bg-black/30 transition-opacity duration-300 data-ending-style:opacity-0 data-starting-style:opacity-0 motion-reduce:transition-none',
        className
      )}
      {...props}
    />
  );
}

function MobileSheetClose({ className, children, render, ...props }: DrawerPrimitive.Close.Props) {
  return (
    <DrawerPrimitive.Close
      data-slot="mobile-sheet-close"
      className={cn(className)}
      render={render ?? <Button variant="secondary" size="icon-sm" />}
      {...props}
    >
      {children ?? (
        <>
          <IconX />
          <span className="sr-only">Close</span>
        </>
      )}
    </DrawerPrimitive.Close>
  );
}

function MobileSheetLink({ to, className, onClick, ...props }: ComponentProps<typeof Button> & { to: string }) {
  const { push } = useMobileSheet();

  return (
    <Button
      data-slot="mobile-sheet-link"
      className={cn(className)}
      {...props}
      type="button"
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          push(to);
        }
      }}
    />
  );
}

function MobileSheetBack({ className, onClick, ...props }: ComponentProps<typeof Button>) {
  const { back } = useMobileSheet();

  return (
    <Button
      data-slot="mobile-sheet-back"
      className={cn(className)}
      {...props}
      type="button"
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          back();
        }
      }}
    />
  );
}

export const MobileSheet = {
  Root: MobileSheetRoot,
  Trigger: MobileSheetTrigger,
  Content: MobileSheetContent,
  Close: MobileSheetClose,
  Link: MobileSheetLink,
  Back: MobileSheetBack,
};
