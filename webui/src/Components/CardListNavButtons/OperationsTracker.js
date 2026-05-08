import React from "react";
import { useOperations } from "../../OperationsContext";

export default function OperationsTracker() {
  const ops = useOperations();
  const activeOperations = Object.entries(ops.operations ?? {});
  const operationLogs = ops.operationLogs ?? [];
  const debugEnabled = ops.debugEnabled ?? false;

  return (
    <div className="operations-tracker">
      <div className="operation-active-badges">
        <span className="badge bg-secondary badge-primary">
          {activeOperations.length} operations active
        </span>
        {activeOperations.map(([key, o]) => (
          <span key={key} className="badge bg-secondary badge-primary">
            {o.message}
          </span>
        ))}
      </div>
      {debugEnabled && (
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
      )}
    </div>
  );
}
