import { createContext, useState, useContext } from "react";

export const ModeContext = createContext({ mode: "full", collectionsEnabled: false });

export function ModeProvider({ children, mode = "full", collectionsEnabled = false }) {
  return (
    <ModeContext.Provider value={{ mode, collectionsEnabled }}>{children}</ModeContext.Provider>
  );
}

export function useMode() {
  return useContext(ModeContext);
}

export const OperationsContext = createContext({});

export function OperationsProvider({ children }) {
  const [operations, setOperations] = useState({});

  const addOperation = (key, operation) => {
    setOperations((prev) => {
      return { ...prev, [key]: operation };
    });
  };

  const removeOperation = (key) => {
    setOperations((prev) => {
      const copy = { ...prev };
      delete copy[key];
      return copy;
    });
  };

  const opsFetch = async (message, defaultValue, ...args) => {
    let opId = crypto.randomUUID?.() ?? Math.random().toString(36).slice(2);
    addOperation(opId, { message: message });
    try {
      const response = await fetch(...args);
      if (response.ok) {
        const result = await response.json();
        removeOperation(opId);
        return result;
      } else {
        let errorMessage = `Request failed (${response.status})`;
        try {
          const body = await response.json();
          if (body?.error) errorMessage = body.error;
        } catch (_) {}
        removeOperation(opId);
        throw new Error(errorMessage);
      }
    } catch (e) {
      removeOperation(opId);
      throw e;
    }
  };

  return (
    <OperationsContext.Provider
      value={{ operations: operations, fetch: opsFetch }}
    >
      {children}
    </OperationsContext.Provider>
  );
}

export function useOperations() {
  return useContext(OperationsContext);
}
