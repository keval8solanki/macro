import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

function App() {
  const [view, setView] = useState('dashboard');

  return (
    <div className="container">
      <header>
        <h1>Macro</h1>
        <p className="subtitle">Advanced Event Replay</p>
      </header>

      <nav className="nav-bar">
        <button onClick={() => setView('dashboard')} className={view === 'dashboard' ? 'active' : ''}>Dashboard</button>
        <button onClick={() => setView('record')} className={view === 'record' ? 'active' : ''}>Record</button>
        <button onClick={() => setView('play')} className={view === 'play' ? 'active' : ''}>Play</button>
      </nav>

      <main className="content">
        {view === 'dashboard' && <Dashboard setView={setView} />}
        {view === 'record' && <Recorder />}
        {view === 'play' && <Player />}
      </main>
    </div>
  );
}

function Dashboard({ setView }) {
  return (
    <div className="card dashboard-card">
      <h2>Welcome Back</h2>
      <p>Your automated workflow awaits.</p>
      <div className="action-grid">
        <div className="action-item" onClick={() => setView('record')}>
          <div className="icon">🔴</div>
          <h3>New Recording</h3>
          <p>Capture mouse and keyboard events</p>
        </div>
        <div className="action-item" onClick={() => setView('play')}>
          <div className="icon">▶️</div>
          <h3>Playback</h3>
          <p>Run your saved macros</p>
        </div>
      </div>
    </div>
  );
}

function Recorder() {
  const [recording, setRecording] = useState(false);
  const [filename, setFilename] = useState('');
  const [status, setStatus] = useState('Ready');

  const startRecording = async () => {
    if (!filename) {
      setStatus('Please enter a filename');
      return;
    }
    try {
      await invoke('start_recording', { filename });
      setRecording(true);
      setStatus('Recording...');
    } catch (e) {
      console.error(e);
      setStatus('Error starting recording');
    }
  };

  const stopRecording = async () => {
    try {
      await invoke('stop_recording');
      setRecording(false);
      setStatus('Recording saved!');
    } catch (e) {
      console.error(e);
      setStatus('Error stopping recording');
    }
  };

  return (
    <div className="card recorder-card">
      <h2>Recorder</h2>
      <div className="input-group">
        <label>Macro Name</label>
        <input
          type="text"
          placeholder="e.g., daily-login"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
          disabled={recording}
        />
      </div>

      <div className="status-display">
        {status}
      </div>

      <div className="controls">
        {!recording ? (
          <button onClick={startRecording} className="btn-primary">Start Recording</button>
        ) : (
          <button onClick={stopRecording} className="btn-danger pulse">Stop Recording</button>
        )}
      </div>
    </div>
  );
}

function Player() {
  const [files, setFiles] = useState([]);
  const [selectedFile, setSelectedFile] = useState(null);
  const [status, setStatus] = useState('');

  useEffect(() => {
    loadFiles();
  }, []);

  const loadFiles = async () => {
    try {
      const recordings = await invoke('get_recordings');
      setFiles(recordings);
    } catch (e) {
      console.error(e);
      setStatus('Error loading files');
    }
  };

  const playRecording = async () => {
    if (!selectedFile) return;
    try {
      setStatus('Playing...');
      await invoke('play_macro', { filename: selectedFile });
      setStatus('Playback started');
      setTimeout(() => setStatus(''), 3000);
    } catch (e) {
      console.error(e);
      setStatus('Error playing macro');
    }
  };

  return (
    <div className="card player-card">
      <h2>Library</h2>
      <div className="file-list">
        {files.length === 0 ? (
          <p className="empty-state">No recordings found</p>
        ) : (
          files.map(file => (
            <div
              key={file}
              className={`file-item ${selectedFile === file ? 'selected' : ''}`}
              onClick={() => setSelectedFile(file)}
            >
              <span className="file-icon">📄</span>
              <span className="file-name">{file}</span>
            </div>
          ))
        )}
      </div>

      {status && <div className="status-display">{status}</div>}

      <div className="controls">
        <button
          disabled={!selectedFile}
          onClick={playRecording}
          className="btn-primary"
        >
          Play Selected
        </button>
        <button onClick={loadFiles} className="btn-secondary">Refresh</button>
      </div>
    </div>
  );
}

export default App;
