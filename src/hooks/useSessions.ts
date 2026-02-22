import { useCallback, useEffect, useState } from "react";
import { analyzeSession, getSessionDetails, listSessions, startRecording, stopRecording } from "../lib/tauriApi";
import type { AnalysisResult, Session, SessionDetails } from "../types";

export function useSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [details, setDetails] = useState<SessionDetails | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refreshSessions = useCallback(async () => {
    const all = await listSessions();
    setSessions(all);
    if (!selectedId && all.length > 0) {
      setSelectedId(all[0].id);
    }
  }, [selectedId]);

  const refreshDetails = useCallback(async (sessionId: string) => {
    const payload = await getSessionDetails(sessionId);
    setDetails(payload);
  }, []);

  useEffect(() => {
    void (async () => {
      setLoading(true);
      setError(null);
      try {
        await refreshSessions();
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load sessions");
      } finally {
        setLoading(false);
      }
    })();
  }, [refreshSessions]);

  useEffect(() => {
    if (!selectedId) {
      setDetails(null);
      return;
    }
    void (async () => {
      setLoading(true);
      setError(null);
      try {
        await refreshDetails(selectedId);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load session details");
      } finally {
        setLoading(false);
      }
    })();
  }, [selectedId, refreshDetails]);

  const onStartRecording = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await startRecording();
      await refreshSessions();
      setSelectedId(res.session.id);
      await refreshDetails(res.session.id);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to start recording");
    } finally {
      setLoading(false);
    }
  }, [refreshSessions, refreshDetails]);

  const onStopRecording = useCallback(async () => {
    const recordingSession = sessions.find((session) => session.status === "recording");
    if (!recordingSession) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const res = await stopRecording(recordingSession.id);
      await refreshSessions();
      setSelectedId(res.session.id);
      await refreshDetails(res.session.id);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to stop recording");
    } finally {
      setLoading(false);
    }
  }, [sessions, refreshSessions, refreshDetails]);

  const onAnalyze = useCallback(
    async (sessionId: string): Promise<AnalysisResult | null> => {
      setLoading(true);
      setError(null);
      try {
        const res = await analyzeSession(sessionId);
        await refreshSessions();
        await refreshDetails(sessionId);
        return res;
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to analyze session");
        return null;
      } finally {
        setLoading(false);
      }
    },
    [refreshSessions, refreshDetails]
  );

  const recordingSession = sessions.find((session) => session.status === "recording") ?? null;

  return {
    sessions,
    selectedId,
    setSelectedId,
    details,
    loading,
    error,
    recordingSession,
    onStartRecording,
    onStopRecording,
    onAnalyze,
    refreshSessions,
    refreshDetails
  };
}
