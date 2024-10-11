import { useState } from 'react';
import { useSSE, SSEProvider } from 'react-hooks-sse';
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable"

function Game() {
   const state = useSSE('game', {
     "players": {}, "connections": {},
   });
    
   return (<>{JSON.stringify(state)}</>);
}

function Chat() {
   const state = useSSE('chat', []);
    
   return (<>{JSON.stringify(state)}</>);
}


function App() {
  const [count, setCount] = useState(0)

  return (
    <SSEProvider endpoint="/live">
    <ResizablePanelGroup
      direction="horizontal"
      className="rounded-lg border"
    >
      <ResizablePanel defaultSize={50} className="overflow-scroll">
        <div className="flex items-center justify-center p-6 overflow-scroll">
          <span className="font-semibold"><Game/></span>
        </div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize={50} className="overflow-scroll">
        <ResizablePanelGroup direction="vertical">
          <ResizablePanel defaultSize={25}>
            <div className="flex h-full items-center justify-center p-6 overflow-scroll">
              <span className="font-semibold">Two</span>
            </div>
          </ResizablePanel>
          <ResizableHandle />
          <ResizablePanel defaultSize={75}>
            <div className="flex h-full items-center justify-center p-6">
              <span className="font-semibold"><Chat/></span>
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </ResizablePanel>
    </ResizablePanelGroup>
    </SSEProvider>
  )
}

export default App

