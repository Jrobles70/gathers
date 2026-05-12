import {
  createContext,
  useContext,
  useEffect,
  useReducer,
} from "react";
import { useOperations, useMode } from "../OperationsContext";
import { useLocation, useParams } from "react-router-dom";

export const ALL_COLLECTIONS_ID = "__all__";
export const ALL_COLLECTIONS_LABEL = "All Collections";

export function isAllCollections(collection) {
  return collection === ALL_COLLECTIONS_ID;
}

export function collectionDisplayName(collection) {
  return isAllCollections(collection) ? ALL_COLLECTIONS_LABEL : collection;
}

export function CollectionsProvider({ children }) {
  const { collection = "Main", pageNumber = 1 } = useParams();
  const location = useLocation();
  const activeCollection = location.pathname.startsWith("/collections")
    ? ALL_COLLECTIONS_ID
    : collection;
  const [collections, collectionsDispatch] = useReducer(collectionsReducer, []);

  const { fetch: opsFetch } = useOperations();
  const { collectionsEnabled } = useMode();

  useEffect(() => {
    if (!collectionsEnabled) return;
    opsFetch("Listing collections", [], "/collection/list").then((data) => {
      collectionsDispatch({
        type: "overwrite",
        collections: data,
      });
    });
  }, [collectionsEnabled, opsFetch]);

  return (
    <CollectionContext.Provider value={activeCollection}>
      <PageNumberContext.Provider value={pageNumber}>
        <CollectionsContext.Provider value={collections}>
          <CollectionsDispatchContext.Provider value={collectionsDispatch}>
            {children}
          </CollectionsDispatchContext.Provider>
        </CollectionsContext.Provider>
      </PageNumberContext.Provider>
    </CollectionContext.Provider>
  );
}

const CollectionContext = createContext("Main");
export function useCollection() {
  return useContext(CollectionContext);
}

const PageNumberContext = createContext(0);
export function usePageNumber() {
  return useContext(PageNumberContext);
}

const CollectionsContext = createContext([]);
export function useCollections() {
  return useContext(CollectionsContext);
}

const CollectionsDispatchContext = createContext(null);
export function useCollectionsDispatch() {
  return useContext(CollectionsDispatchContext);
}

function collectionsReducer(collections, action) {
  switch (action.type) {
    case "added": {
      const exists = collections.some((collection) => collection.id === action.item.id);
      return exists ? collections : [...collections, action.item];
    }
    case "addrange": {
      return [...collections, ...action.collections];
    }
    case "overwrite": {
      return [...action.collections];
    }
    case "deleted": {
      return collections.filter((t) => t.id !== action.id);
    }
    case "renamed": {
      return collections.map((collection) =>
        collection.id === action.from ? action.item : collection,
      );
    }
    case "updated": {
      return collections.map((collection) =>
        collection.id === action.item.id ? action.item : collection,
      );
    }
    default: {
      throw Error("Unknown action: " + action.type);
    }
  }
}
