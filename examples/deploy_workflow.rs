use futures::executor::block_on;
use zeebest::Client;

fn main() {
    let client = Client::new("127.0.0.1", 26500).unwrap();

    let result = block_on(client.deploy_bpmn_workflow("simple-process", SIMPLE_PROCESS_XML.into()));

    println!("deploy workflow result: {:?}", result);
}

const SIMPLE_PROCESS_XML: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" xmlns:di="http://www.omg.org/spec/DD/20100524/DI" xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" xmlns:zeebe="http://camunda.org/schema/zeebe/1.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" id="Definitions_1" targetNamespace="http://bpmn.io/schema/bpmn" exporter="Zeebe Modeler" exporterVersion="0.3.0">
  <bpmn:process id="simple-process" isExecutable="true">
    <bpmn:startEvent id="order-placed" name="Order Placed">
      <bpmn:outgoing>SequenceFlow_18tqka5</bpmn:outgoing>
    </bpmn:startEvent>
    <bpmn:endEvent id="order-delivered" name="Order Delivered">
      <bpmn:incoming>SequenceFlow_0ei7pxo</bpmn:incoming>
    </bpmn:endEvent>
    <bpmn:sequenceFlow id="SequenceFlow_18tqka5" sourceRef="order-placed" targetRef="collect-money" />
    <bpmn:serviceTask id="collect-money" name="Collect Money">
      <bpmn:extensionElements>
        <zeebe:taskDefinition type="payment-service" />
        <zeebe:taskHeaders>
          <zeebe:header key="method" value="VISA" />
        </zeebe:taskHeaders>
        <zeebe:ioMapping outputBehavior="">
          <zeebe:input source="orderId" target="orderId" />
        </zeebe:ioMapping>
      </bpmn:extensionElements>
      <bpmn:incoming>SequenceFlow_18tqka5</bpmn:incoming>
      <bpmn:outgoing>SequenceFlow_10zt7r3</bpmn:outgoing>
    </bpmn:serviceTask>
    <bpmn:sequenceFlow id="SequenceFlow_10zt7r3" sourceRef="collect-money" targetRef="IntermediateCatchEvent_0tyjtus" />
    <bpmn:intermediateCatchEvent id="IntermediateCatchEvent_0tyjtus" name="Payment Confirmed">
      <bpmn:incoming>SequenceFlow_10zt7r3</bpmn:incoming>
      <bpmn:outgoing>SequenceFlow_0ei7pxo</bpmn:outgoing>
      <bpmn:messageEventDefinition messageRef="Message_0n22whn" />
    </bpmn:intermediateCatchEvent>
    <bpmn:sequenceFlow id="SequenceFlow_0ei7pxo" sourceRef="IntermediateCatchEvent_0tyjtus" targetRef="order-delivered" />
  </bpmn:process>
  <bpmn:message id="Message_0n22whn" name="payment-confirmed">
    <bpmn:extensionElements>
      <zeebe:subscription correlationKey="orderId" />
    </bpmn:extensionElements>
  </bpmn:message>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="simple-process">
      <bpmndi:BPMNShape id="_BPMNShape_StartEvent_2" bpmnElement="order-placed">
        <dc:Bounds x="200" y="102" width="36" height="36" />
        <bpmndi:BPMNLabel>
          <dc:Bounds x="186" y="138" width="65" height="14" />
        </bpmndi:BPMNLabel>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNShape id="EndEvent_1253stq_di" bpmnElement="order-delivered">
        <dc:Bounds x="559" y="102" width="36" height="36" />
        <bpmndi:BPMNLabel>
          <dc:Bounds x="538" y="141" width="78" height="14" />
        </bpmndi:BPMNLabel>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNEdge id="SequenceFlow_18tqka5_di" bpmnElement="SequenceFlow_18tqka5">
        <di:waypoint x="236" y="120" />
        <di:waypoint x="294" y="120" />
        <bpmndi:BPMNLabel>
          <dc:Bounds x="251.5" y="98.5" width="0" height="13" />
        </bpmndi:BPMNLabel>
      </bpmndi:BPMNEdge>
      <bpmndi:BPMNShape id="ServiceTask_0298fyo_di" bpmnElement="collect-money">
        <dc:Bounds x="294" y="80" width="100" height="80" />
      </bpmndi:BPMNShape>
      <bpmndi:BPMNEdge id="SequenceFlow_10zt7r3_di" bpmnElement="SequenceFlow_10zt7r3">
        <di:waypoint x="394" y="120" />
        <di:waypoint x="464" y="120" />
        <bpmndi:BPMNLabel>
          <dc:Bounds x="426" y="98" width="0" height="13" />
        </bpmndi:BPMNLabel>
      </bpmndi:BPMNEdge>
      <bpmndi:BPMNShape id="IntermediateCatchEvent_0tyjtus_di" bpmnElement="IntermediateCatchEvent_0tyjtus">
        <dc:Bounds x="464" y="102" width="36" height="36" />
        <bpmndi:BPMNLabel>
          <dc:Bounds x="457" y="145" width="51" height="27" />
        </bpmndi:BPMNLabel>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNEdge id="SequenceFlow_0ei7pxo_di" bpmnElement="SequenceFlow_0ei7pxo">
        <di:waypoint x="500" y="120" />
        <di:waypoint x="559" y="120" />
      </bpmndi:BPMNEdge>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#;
