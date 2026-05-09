import React from "react";
import { useOperations } from "../../OperationsContext";

export default function OperationsTracker() {
  const ops = useOperations();
  const operationLogs = ops.operationLogs ?? [];
  const debugEnabled = ops.debugEnabled ?? false;

  if (!debugEnabled) return null;

  return (
    <div className="operations-tracker">
      <div className="operation-log" aria-label="Operation logs">
        {operationLogs.length === 0 ? (
          <div className="operation-log-empty">No operations logged</div>
        ) : (
          operationLogs.map((log) => (
            <div key={log.id} className={`operation-log-row ${log.status}`}>
              <span className="operation-log-time">{log.time}</span>
              <span className="operation-log-message">{log.message}</span>
              <span className="operation-log-status">{log.status}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
