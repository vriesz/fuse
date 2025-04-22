# experiments/analysis/visualize.py
import pandas as pd
import seaborn as sns

def plot_tradeoffs(df):
    """Generate 3D Pareto frontier plots"""
    fig = px.scatter_3d(df, 
        x='power_w', 
        y='processing_tflops', 
        z='detection_accuracy',
        color='architecture_type')
    fig.show()