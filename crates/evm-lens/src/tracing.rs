use pretty::{Doc, RcDoc};

#[derive(Debug, Clone)]
enum TraceEvent {
   Call { ident: String, call_depth: u64 },
   Return { data: String, call_depth: u64 },
}

impl TraceEvent {
   fn call_depth(&self) -> u64 {
       match self {
           TraceEvent::Call { call_depth, .. } => *call_depth,
           TraceEvent::Return { call_depth, .. } => *call_depth,
       }
   }
}

fn generate_prefix(depth: u64, is_connector: bool) -> String {
   if depth == 0 {
       return String::new();
   }
   
   let mut prefix = String::new();
   
   // Add vertical bars for each parent level
   for level in 0..depth {
       if level == depth - 1 {
           // This is the current level
           if is_connector {
               prefix.push_str("|");
           } else {
               prefix.push_str("|_ ");
           }
       } else {
           // Parent levels get continuation bars
           prefix.push_str("|     ");
       }
   }
   
   prefix
}

fn render_trace_event(event: &TraceEvent) -> Vec<RcDoc<'static>> {
   let depth = event.call_depth();
   let mut docs = Vec::new();
   
   // Add connector lines if depth > 0
   if depth > 0 {
       docs.push(RcDoc::text(generate_prefix(depth, true)));
   }
   
   // Add the actual call/return
   match event {
       TraceEvent::Call { ident, .. } => {
           docs.push(RcDoc::text(format!("{}{}", generate_prefix(depth, false), ident)));
       }
       TraceEvent::Return { data, .. } => {
           // Returns also get a connector line first
           if depth > 0 {
               docs.push(RcDoc::text(generate_prefix(depth, true)));
           }
           docs.push(RcDoc::text(format!("{}-> {}", generate_prefix(depth, false), data)));
       }
   }
   
   docs
}

fn render_trace(events: &[TraceEvent]) -> RcDoc<'static> {
   let mut all_docs = Vec::new();
   
   for event in events {
       let event_docs = render_trace_event(event);
       all_docs.extend(event_docs);
   }
   

   RcDoc::intersperse(all_docs, Doc::line())
}

#[cfg(test)]

mod test {
use super::*;

    #[test]
    fn test_structred_tracing() {
        let events = vec![
            TraceEvent::Call { ident: "a.call()".to_string(), call_depth: 0 },
            TraceEvent::Call { ident: "b.call()".to_string(), call_depth: 1 },
            TraceEvent::Return { data: "42".to_string(), call_depth: 1 },
       TraceEvent::Call { ident: "c.call()".to_string(), call_depth: 1 },
       TraceEvent::Call { ident: "d.call()".to_string(), call_depth: 2 },
       TraceEvent::Return { data: "hello".to_string(), call_depth: 2 },
       TraceEvent::Return { data: "world".to_string(), call_depth: 1 },
       TraceEvent::Return { data: "done".to_string(), call_depth: 0 },
       ];
       
       let doc = render_trace(&events);
       let formatted = doc.pretty(80);
       println!("{}", formatted);
    }
}