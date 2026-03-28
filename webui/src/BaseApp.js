import React from "react";
import { OperationsProvider, ModeProvider } from "./OperationsContext";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import CardListView from "./Views/CardListView";
import SearchView from "./Views/SearchView";

export default function BaseApp({ mode = "full", collectionsEnabled = false }) {
  return (
    <ModeProvider mode={mode} collectionsEnabled={collectionsEnabled}>
      <OperationsProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<Navigate to="/search" />} />
            <Route path="/search" element={<SearchView />} />
            {collectionsEnabled ? (
              <>
                <Route path="/c/:collection" element={<CardListView />} />
                <Route
                  path="/c/:collection/:pageNumber"
                  element={<CardListView />}
                />
              </>
            ) : (
              <Route path="/c/*" element={<Navigate to="/search" />} />
            )}
          </Routes>
        </BrowserRouter>
      </OperationsProvider>
    </ModeProvider>
  );
}
