

import { screen, within } from '@testing-library/react';
import { renderComponentRoute } from './renderUtils';

export async function runLinkVisiblityTest(
    routeName,
    heading,
    expectedMenu
) {
  const expectedLinkText = expectedMenu.text + ' ' + expectedMenu.description;

  await renderComponentRoute(routeName);

  const main = await assertPageRendered(heading);

  const list = within(main).getByRole('list');
  expect(list).toBeVisible();

  expect(within(list)
      .getByRole('link', { name: expectedLinkText })
  ).toBeVisible();
}

export async function runLinkNotVisibleTest(routeName, heading, expectedMenu) {
  const expectedLinkText = expectedMenu.text + ' ' + expectedMenu.description;

  await renderComponentRoute(routeName);

  const main = await assertPageRendered(heading);

  expect(within(main)
      .queryByRole('link', { name: expectedLinkText })
  ).not.toBeInTheDocument();
}

export async function runDirectoryPage404Test(routeName) {
  await renderComponentRoute(routeName);

  // MissingRoutePage (Preview): no <main>; 404 code is plain text, title is h1
  expect(screen.getByText('404')).toBeVisible();
  expect(screen.getByRole('heading', { name: /resource not found/i })).toBeVisible();
}

async function assertPageRendered(heading) {
  const main = await screen.getByRole('main');
  within(main).getByRole('heading', { name: heading })

  return main;
}
