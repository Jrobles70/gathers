# Quick Search Card Controls Plan

## Goals

- Keep quick search card titles from covering quantity controls by removing the hover name link layer.
- Make the card image the details-page link.
- Return from a details page to the originating search state, including query parameters and page number.
- Replace separate regular and foil add buttons with a compact quantity rail and a foil mode toggle in the card menu.
- Reserve a visible price row below each card using `$-` until price data is available.
- Collapse duplicate search printings by card name and offer a printing picker from the card menu.

## Implementation Notes

- Card image clicks should use route state to preserve the intended return path.
- Quick search links should return to `/search` with the current search parameters rather than the collection route that hosts the modal.
- Quantity controls should post to the existing collection add/delete endpoints and update either regular or foil quantities based on the menu toggle.
- The menu should hold secondary card actions, including foil mode and printing selection.
- Printing grouping can happen on the search results returned for the current page. A broader backend search endpoint can later provide all printings for a card name if needed.
- Price display should be a stable footer element so real price data can replace the `$-` placeholder without changing layout.

## Test Plan

- Unit test collection resolution and card control rendering.
- Unit test printing grouping and detail return path helpers.
- Run the React test suite in watch-disabled mode.
- Run a production build to catch routing, lint, and bundling regressions.

