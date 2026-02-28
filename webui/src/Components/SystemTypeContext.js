import { createContext, useContext, useState, useEffect } from "react";
import { useOperations } from "../OperationsContext";

const SystemTypeContext = createContext(null);

export function useSystemType() {
  return useContext(SystemTypeContext);
}

export function SystemTypeProvider({ children }) {
  const ops = useOperations();
  const [systemType, setSystemType] = useState(null);

  useEffect(() => {
    ops
      .fetch("Getting system info", null, "/system", {})
      .then((r) => {
        if (r && r.system) {
          setSystemType(r.system);
        } else {
          setSystemType("Sql");
        }
      })
      .catch(() => {
        setSystemType("Sql");
      });
  }, []);

  return (
    <SystemTypeContext.Provider value={systemType}>
      {children}
    </SystemTypeContext.Provider>
  );
}
