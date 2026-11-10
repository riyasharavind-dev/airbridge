import React, {useEffect, useState} from 'react';
import {createRoot} from 'react-dom/client';
import {invoke} from '@tauri-apps/api/core';
type Peer={address:string;name:string;os:string};
type Diagnostics = {os:string;architecture:string;device_name:string;session_type:string};
function App(){
 const [peers,setPeers]=useState<Peer[]>([]);const [discoveryError,setDiscoveryError]=useState('');
 const [data,setData]=useState<Diagnostics>();const [error,setError]=useState('');const [loading,setLoading]=useState(false);
 async function refresh(){setLoading(true);setError('');try{setData(await invoke<Diagnostics>('diagnostics'));}catch(e){setError(String(e));}finally{setLoading(false);}}
 useEffect(()=>{void refresh(); async function scan(){try{setPeers(await invoke<Peer[]>('nearby_devices'));setDiscoveryError('')}catch(e){setDiscoveryError(String(e))}}void scan();const timer=setInterval(()=>void scan(),3000);return ()=>clearInterval(timer)},[]);
 return <main style={{fontFamily:'system-ui',padding:48,background:'#101827',color:'#eef2ff',minHeight:'85vh'}}><p>AIRBRIDGE / DEVELOPMENT BUILD</p><h1>Your screens. Connected.</h1><p>Native wireless display for Windows and Linux.</p><h2>This device</h2>{data&&<dl>{Object.entries(data).map(([key,value])=><React.Fragment key={key}><dt style={{color:'#6ee7d0',marginTop:16}}>{key.replaceAll('_',' ')}</dt><dd style={{margin:0}}>{value}</dd></React.Fragment>)}</dl>}{error&&<p role="alert">Could not inspect this device: {error}</p>}<button onClick={()=>void refresh()} disabled={loading}>{loading?'Checking…':'Refresh device details'}</button><h2>LAN discovery</h2>{discoveryError?<p role="alert">{discoveryError}</p>:peers.length?<ul>{peers.map(peer=><li key={peer.address}>{peer.name} · {peer.os} · {peer.address}</li>)}</ul>:<p>No advertisements received. Open AirBridge on another computer on the same LAN.</p>}<p>Advertisements are unverified; they do not authorize a connection. This device may also appear.</p><h2>Connection readiness</h2><p>Secure pairing and mirroring are under development.</p><p>Extended display is unavailable until a genuine virtual monitor backend is verified.</p></main>
}
createRoot(document.getElementById('root')!).render(<App/>);
