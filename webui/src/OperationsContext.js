import { createContext, useState, useContext, useCallback } from "react";

export const ModeContext = createContext({ mode: "full", collectionsEnabled: false });

export function ModeProvider({ children, mode = "full", collectionsEnabled = false }) {
  return (
    <ModeContext.Provider value={{ mode, collectionsEnabled }}>{children}</ModeContext.Provider>
  );
}

export function useMode() {
  return useContext(ModeContext);
}

const MAX_OPERATION_LOGS = 20;

export function formatOperationTime(date = new Date()) {
  return `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
}

export function prependOperationLog(logs, entry) {
  return [entry, ...logs].slice(0, MAX_OPERATION_LOGS);
}

export function updateOperationLog(logs, id, updates) {
  return logs.map((log) => (log.id === id ? { ...log, ...updates } : log));
}

export const OperationsContext = createContext({});

export function OperationsProvider({ children }) {
  const [operations, setOperations] = useState({});
  const [operationLogs, setOperationLogs] = useState([]);
  const [debugEnabled, setDebugEnabled] = useState(false);

  const opsFetch = useCallback(async (message, defaultValue, ...args) => {
    const opId = crypto.randomUUID?.() ?? Math.random().toString(36).slice(2);
    setOperations((prev) => ({ ...prev, [opId]: { message } }));
    setOperationLogs((prev) =>
      prependOperationLog(prev, {
        id: opId,
        message,
        status: "active",
        time: formatOperationTime(),
      }),
    );

    const removeOp = () => setOperations((prev) => { const copy = { ...prev }; delete copy[opId]; return copy; });
    const finishOp = (status) => {
      removeOp();
      setOperationLogs((prev) => updateOperationLog(prev, opId, { status }));
    };

    try {
      const response = await fetch(...args);
      if (!response.ok) {
        let errorMessage = `Request failed (${response.status})`;
        try {
          const body = await response.json();
          if (body?.error) errorMessage = body.error;
        } catch (_) {}
        throw new Error(errorMessage);
      }
      const result = await response.json();
      finishOp("done");
      return result;
    } catch (e) {
      finishOp("failed");
      throw e;
    }
  }, []);

  return (
    <OperationsContext.Provider
      value={{
        operations: operations,
        operationLogs,
        debugEnabled,
        setDebugEnabled,
        fetch: opsFetch,
      }}
    >
      {children}
    </OperationsContext.Provider>
  );
}

export function useOperations() {
  return useContext(OperationsContext);
}
