import { create } from 'zustand';

interface TextState {
  inputText: string;
  algorithm: string;
  result: Record<string, unknown> | null;
  loading: boolean;
  setInputText: (v: string) => void;
  setAlgorithm: (v: string) => void;
  setResult: (v: Record<string, unknown> | null) => void;
  setLoading: (v: boolean) => void;
  reset: () => void;
}

export const useTextStore = create<TextState>((set) => ({
  inputText: '',
  algorithm: 'hybrid',
  result: null,
  loading: false,
  setInputText: (inputText) => set({ inputText }),
  setAlgorithm: (algorithm) => set({ algorithm }),
  setResult: (result) => set({ result }),
  setLoading: (loading) => set({ loading }),
  reset: () => set({ inputText: '', algorithm: 'hybrid', result: null, loading: false }),
}));
