import { createRouter } from '@tanstack/react-router';

import { routeTree } from './routeTree.gen';

const pageOrder = ['/', '/settings'];

export const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
  scrollRestoration: true,
  defaultViewTransition: {
    types: ({ fromLocation, toLocation }) => {
      if (!fromLocation) {
        return false;
      }

      const fromIndex = pageOrder.indexOf(fromLocation.pathname);
      const toIndex = pageOrder.indexOf(toLocation.pathname);

      if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
        return false;
      }

      return [fromIndex < toIndex ? 'slide-left' : 'slide-right'];
    },
  },
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
