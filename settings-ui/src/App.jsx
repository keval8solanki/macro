import { useState, useEffect } from 'react'
import { Button, Form, InputNumber, ConfigProvider, theme, message, Tooltip } from 'antd'
import { ReloadOutlined } from '@ant-design/icons'
import './App.css'

function App() {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    try {
      const config = window.initialConfig || { speed: 1.0, repeat: 1, interval: 0.0 };

      form.setFieldsValue({
        speed: config.speed,
        repeat: config.repeat,
        interval: config.interval
      });
      setLoading(false);
    } catch (e) {
      console.error("Error loading config:", e);
      setLoading(false);
    }
  }, [form]);

  const onFinish = (values) => {
    const settings = {
      speed: values.speed,
      repeat: values.repeat,
      interval: values.interval
    };

    if (window.ipc) {
      window.ipc.postMessage(JSON.stringify(settings));
    } else {
      console.log("Settings applied:", settings);
      message.success("Settings applied (Dev Mode)");
    }
  };

  const handleReset = () => {
    form.setFieldsValue({
      speed: 1.0,
      repeat: 1,
      interval: 0.0
    });
    message.info("Settings reset to defaults");
  };

  if (loading) return <div style={{ color: 'white', padding: 20 }}>Loading...</div>;

  return (
    <ConfigProvider
      theme={{
        algorithm: theme.darkAlgorithm,
        token: {
          colorPrimary: '#ffffff', // White Accent
          borderRadius: 30, // Maximum radius for pill shape
        },
        components: {
          Form: {
            itemMarginBottom: 12,
            labelFontSize: 13,
          },
          InputNumber: {
            controlHeight: 30,
          },
          Button: {
            controlHeight: 32,
            primaryColor: '#000000', // Black text on white button
            colorPrimary: '#ffffff', // White background
            loginHoverBg: '#e6e6e6', // Slightly gray on hover
          }
        }
      }}
    >
      <div style={{ height: '100vh', padding: 16, display: 'flex', flexDirection: 'column', position: 'relative' }}>

        <Tooltip title="Reset">
          <Button
            type="text"
            icon={<ReloadOutlined />}
            onClick={handleReset}
            style={{
              position: 'absolute',
              top: 10,
              right: 10,
              color: '#666',
              zIndex: 10
            }}
          />
        </Tooltip>

        <Form
          form={form}
          layout="vertical"
          onFinish={onFinish}
          initialValues={{ speed: 1.0, repeat: 1, interval: 0 }}
          style={{ height: '100%', display: 'flex', flexDirection: 'column', marginTop: 10 }}
        >
          <Form.Item
            label="Speed"
            name="speed"
            rules={[{ required: true, message: 'Required' }]}
          >
            <InputNumber
              step={0.1}
              min={0.1}
              style={{ width: '100%' }}
            />
          </Form.Item>

          <Form.Item
            label="Repeat (0 = Infinite)"
            name="repeat"
            rules={[{ required: true, message: 'Required' }]}
          >
            <InputNumber
              min={0}
              step={1}
              style={{ width: '100%' }}
            />
          </Form.Item>

          <Form.Item
            label="Interval"
            name="interval"
          >
            <InputNumber
              step={0.1}
              min={0.0}
              style={{ width: '100%' }}
            />
          </Form.Item>

          <div style={{ marginTop: 'auto' }}>
            <Button type="primary" htmlType="submit" block style={{ color: 'black', fontWeight: 600 }}>
              Apply
            </Button>
          </div>
        </Form>
      </div>
    </ConfigProvider>
  )
}

export default App
