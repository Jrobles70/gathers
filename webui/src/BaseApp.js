import React from "react";
import { OperationsProvider, ModeProvider } from "./OperationsContext";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import CardListView from "./Views/CardListView";
import SearchView from "./Views/SearchView";

export default function BaseApp({ mode = "full" }) {
  const isSearchOnly = mode === "search-only";
  return (
    <ModeProvider mode={mode}>
      <OperationsProvider>
        <BrowserRouter>
          <Routes>
            <Route
              path="/"
              element={
                isSearchOnly ? <Navigate to="/search" /> : <CardListView />
              }
            />
            <Route path="/search" element={<SearchView />} />
            <Route path="/c/:collection" element={<CardListView />} />
            <Route
              path="/c/:collection/:pageNumber"
              element={<CardListView />}
            />
          </Routes>
        </BrowserRouter>
      </OperationsProvider>
    </ModeProvider>
  );
}
